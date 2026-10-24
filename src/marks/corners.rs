//! `make_masks`' corner tables (`g_marksCornersA` / `B`, 0x81 entries of two corners;
//! IDA, 2026-10-17): shape set 0 turned at 0x81 eased steps, each list's farthest pair
//! taken as an entry's two corners, ordered, scaled into the kind's box, flipped and held
//! (`clamp_pair`); `span_ring` makes the rings of them.

use crate::errors::{BadInput, Refused};
use crate::marks::shape_points::{FARTHEST_RVA, TURNED_RVA, turn_shape_checks, turn_shape_points};
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::marks::turns::{clamp_pair, smooth_pair, smooth_steps};
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};

pub const CORNERS_RVA: [Slot; 2] = [Slot::at(0x82E426), Slot::at(0x82F03E)]; // g_marksCornersA, _B
pub const ENTRIES: u32 = 0x81;
const ENTRY: u32 = 0x18;
/// Each table's two steps' starts and the end they both go to.
const STEPS: [([Slot; 2], Slot); 2] = [
    ([Slot::at(0x89C9CC), Slot::at(0x89C9D4)], Slot::at(0x884FC4)),
    ([Slot::at(0x89C9DC), Slot::at(0x89C9E4)], Slot::at(0x884FC8)),
];
const LIFT: u32 = 0xB2A7;
const BOXES_RVA: Slot = Slot::at(0x811BEB); // the kinds' 0x30-byte records
const SCALE_RVA: Slot = Slot::at(0x82E38E); // flt_82E38E, flt_82E392
const DIVISORS_RVA: Slot = Slot::at(0x89C9EC); // dword_89C9EC, dword_89C9F0
const FLIPS_RVA: Slot = Slot::at(0x89C9F4); // dword_89C9F4, dword_89C9F8
const OFFSETS_RVA: [Slot; 2] = [Slot::at(0x8859F4), Slot::at(0x885A00)]; // the block's lowest points

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) }
}

/// The two turns step k gives table n: the second result of each of its two steps.
fn turns(n: usize, k: u32) -> (u32, u32) {
    unsafe {
        let (starts, end) = STEPS[n];
        let b: u32 = game_read(end);
        (
            smooth_pair(k, game_read(starts[0]), b).1,
            smooth_pair(k, game_read(starts[1]), b).1,
        )
    }
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
        let kind: u32 = game_read(crate::marks::kind_values::KIND_RVA);
        if kind >= 0x22 {
            return Err(BadInput("a marks' kind of 0x22 or more").into());
        }
        for f in FARTHEST_RVA {
            let pair: [u32; 2] = game_read(f);
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
        for n in 0..2 {
            let (starts, end) = STEPS[n];
            for k in 0..ENTRIES {
                let first = smooth_steps(k, game_read(starts[0]), game_read(end)).1;
                let second = smooth_steps(k, game_read(starts[1]), game_read(end)).1;
                turn_shape_points(first, second)?;
                let mut to = game_addr(CORNERS_RVA[n]) as u32 + k * ENTRY;
                let pair: [u32; 2] = game_read(FARTHEST_RVA[n]);
                let turned = game_addr(TURNED_RVA[n]) as u32;
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
            for table in CORNERS_RVA {
                let e = game_addr(table) as u32 + k * ENTRY;
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
        let bx_at = game_addr(BOXES_RVA) as u32 + kind * 0x30;
        let bx = |o: u32| get(bx_at + o) as i32;
        let divisors: [i32; 2] = game_read(DIVISORS_RVA);
        let scales = [
            x87::int_div_int(bx(0x24).wrapping_sub(bx(0x18)), divisors[0]),
            x87::int_div_int(bx(4).wrapping_sub(bx(0x1C)), divisors[1]),
        ];
        game_write(SCALE_RVA, scales);
        let corners = 2 * ENTRIES;
        for k in 0..corners {
            for n in 0..2 {
                let e = game_addr(CORNERS_RVA[n]) as u32 + 12 * k;
                let offset: [u32; 2] = game_read(OFFSETS_RVA[n]);
                let x = get(e).wrapping_add(offset[0]).wrapping_sub(bx(0x18) as u32);
                let y = get(e + 4)
                    .wrapping_add(offset[1])
                    .wrapping_sub(bx(0x1C) as u32);
                put(e + 4, x87::int_over(y as i32, scales[1]));
                put(e, x87::int_over(x as i32, scales[0]));
            }
        }
        let flips: [i32; 2] = game_read(FLIPS_RVA);
        for k in 0..corners {
            for n in 0..2 {
                let e = game_addr(CORNERS_RVA[n]) as u32 + 12 * k;
                put(e + 4, x87::int_less(flips[n], get(e + 4)));
            }
        }
        for k in 0..corners {
            for table in CORNERS_RVA {
                let e = game_addr(table) as u32 + 12 * k;
                let out = clamp_pair([extended(get(e + 4)), extended(get(e))]);
                put(e + 4, rounded(out[0]));
                put(e, rounded(out[1]));
            }
        }
    }
    Ok(())
}
