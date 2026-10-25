//! `make_masks`' corner tables (`g_marksCornersA` / `B`, 0x81 entries of two corners;
//! IDA, 2026-10-17): shape set 0 turned at 0x81 eased steps, each list's farthest pair
//! taken as an entry's two corners, ordered, scaled into the kind's box, flipped and held
//! (`clamp_pair`); `span_ring` makes the rings of them.

use crate::errors::{BadInput, Refused};
use crate::marks::edges::KIND_BOXES;
use crate::marks::kind_values::MARKS_KIND;
use crate::marks::mask_constants::MASK_CONSTANTS;
use crate::marks::mask_vars::MASKS;
use crate::marks::shape_points::{farthest_pair, turn_shape_checks, turn_shape_points, turned};
use crate::marks::shape_work::SHAPE_WORK;
use crate::marks::store::game_write;
use crate::marks::turns::{clamp_pair, smooth_pair, smooth_steps};
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::session::{Dword, read_bits, read_first, write_bits};

/// The two corner tables' addresses (`g_marksCornersA`, `_B`).
pub fn tables() -> [u32; 2] {
    [MASKS.corners_a.addr() as u32, MASKS.corners_b.addr() as u32]
}
pub const ENTRIES: u32 = 0x81;
const ENTRY: u32 = 0x18;
/// Each table's two steps' starts and the end they both go to.
static STEPS: [([usize; 2], &Dword); 2] = [
    ([0, 2], &SHAPE_WORK.work.steps[0]), // `MaskConstants::corner_angles`: dword_89C9CC, dword_89C9D4; dword_884FC4
    ([4, 6], &SHAPE_WORK.work.steps[1]), // dword_89C9DC, dword_89C9E4; dword_884FC8
];
const LIFT: u32 = 0xB2A7;
/// The block's lowest points (x, y).
fn offsets(n: usize) -> [u32; 2] {
    let low = if n == 0 {
        &SHAPE_WORK.work.low_a
    } else {
        &SHAPE_WORK.work.low_b
    };
    read_first(low)
}

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) }
}

/// The two turns step k gives table n: the second result of each of its two steps.
fn turns(n: usize, k: u32) -> (u32, u32) {
    let (starts, end) = STEPS[n];
    let b: u32 = end.get();
    (
        smooth_pair(k, MASK_CONSTANTS.corner_angles[starts[0]].bits(), b).1,
        smooth_pair(k, MASK_CONSTANTS.corner_angles[starts[1]].bits(), b).1,
    )
}

/// `marks::make_corners` (0x4BEB25): for each table and each of 0x81 steps k, shape set 0
/// turned (`turn_shape_points`) by the second results of two eased steps (`smooth_steps`),
/// the list's farthest pair copied in as the entry's two corners, y lifted by 0xB2A7; then
/// in each entry the corners swapped (x, y) unless the second is farther from the middle
/// (`fcompp`, `g_fpuStatus`); the scales of the kind's box (its +0x24 - +0x18 over 0x100,
/// +4 - +0x1C over 0x80); each corner (x, y plus the list's lowest point, less the box's
/// +0x18, +0x1C) over them as floats; y from 0x80; and `clamp_pair`. Refuses a kind of
/// 0x22 or more, a farthest pair past 0x3F, what `turn_shape_points` refuses at any step,
/// before writing.
pub unsafe fn make_corners() -> Result<(), Refused> {
    unsafe {
        let kind: u32 = MARKS_KIND.kind.bits();
        if kind >= 0x22 {
            return Err(BadInput("a marks' kind of 0x22 or more").into());
        }
        for n in 0..2 {
            let pair: [u32; 2] = read_bits(farthest_pair(n));
            if pair.iter().any(|&i| i >= 0x40) {
                return Err(BadInput("a farthest pair past 0x3F").into());
            }
        }
        for n in 0..2 {
            for k in 0..ENTRIES {
                let (a, b) = turns(n, k);
                turn_shape_checks(a, b)?;
            }
        }
        for (n, &(starts, end)) in STEPS.iter().enumerate() {
            for k in 0..ENTRIES {
                let first =
                    smooth_steps(k, MASK_CONSTANTS.corner_angles[starts[0]].bits(), end.get()).1;
                let second =
                    smooth_steps(k, MASK_CONSTANTS.corner_angles[starts[1]].bits(), end.get()).1;
                turn_shape_points(first, second)?;
                let mut to = tables()[n] + k * ENTRY;
                let pair: [u32; 2] = read_bits(farthest_pair(n));
                let turned = turned(n).addr() as u32;
                for p in pair {
                    let from = turned + p * 12;
                    for c in 0..3 {
                        put(to + 4 * c, get(from + 4 * c));
                    }
                    put(to + 4, get(to + 4).wrapping_add(LIFT));
                    to += 12;
                }
            }
        }
        for k in 0..ENTRIES {
            for table in tables() {
                let e = table + k * ENTRY;
                let c = |o: u32| get(e + o) as i32;
                let status = x87::compare_squares(c(0), c(4), c(0xC), c(0x10));
                game_write(crate::scene::vector3::STATUS_RVA, status as u32);
                if status != 0 {
                    for (a, b) in [(0, 0xC), (4, 0x10)] {
                        let (x, y) = (get(e + a), get(e + b));
                        put(e + a, y);
                        put(e + b, x);
                    }
                }
            }
        }
        let bx_at = KIND_BOXES.boxes.addr() as u32 + kind * 0x30;
        let bx = |o: u32| get(bx_at + o) as i32;
        let divisors: [i32; 2] = MASK_CONSTANTS.divisors.each_ref().map(|c| c.get() as i32);
        let scales = [
            x87::int_div_int(bx(0x24).wrapping_sub(bx(0x18)), divisors[0]),
            x87::int_div_int(bx(4).wrapping_sub(bx(0x1C)), divisors[1]),
        ];
        write_bits(&MASKS.corner_scale, scales);
        let corners = 2 * ENTRIES;
        for k in 0..corners {
            for n in 0..2 {
                let e = tables()[n] + 12 * k;
                let offset: [u32; 2] = offsets(n);
                let x = get(e).wrapping_add(offset[0]).wrapping_sub(bx(0x18) as u32);
                let y = get(e + 4)
                    .wrapping_add(offset[1])
                    .wrapping_sub(bx(0x1C) as u32);
                put(e + 4, x87::int_over(y as i32, scales[1]));
                put(e, x87::int_over(x as i32, scales[0]));
            }
        }
        let flips: [i32; 2] = MASK_CONSTANTS.flips.each_ref().map(|c| c.get() as i32);
        for k in 0..corners {
            for (n, &flip) in flips.iter().enumerate() {
                let e = tables()[n] + 12 * k;
                put(e + 4, x87::int_less(flip, get(e + 4)));
            }
        }
        for k in 0..corners {
            for table in tables() {
                let e = table + 12 * k;
                let out = clamp_pair([extended(get(e + 4)), extended(get(e))]);
                put(e + 4, rounded(out[0]));
                put(e, rounded(out[1]));
            }
        }
    }
    Ok(())
}
