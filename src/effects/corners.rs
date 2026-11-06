//! `make_masks`' corner tables (`g_marksCornersA` / `B`, 0x81 entries of two corners;
//! IDA, 2026-10-17): shape set 0 turned at 0x81 eased steps, each list's farthest pair
//! taken as an entry's two corners, ordered, scaled into the kind's box, flipped and held
//! (`clamp_pair`); `span_ring` makes the rings of them.

use crate::effects::mask_store::Masks;
use crate::effects::shape_points::{farthest_pair, turn_shape_checks, turn_shape_points, turned};
use crate::effects::turns::{clamp_pair, smooth_pair, smooth_steps};
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::session::{Bytes, Dword, read_bits, read_first, write_bits};

/// The two corner tables (`g_marksCornersA`, `_B`).
pub fn tables(masks: &Masks) -> [&Bytes<3096>; 2] {
    [&masks.vars.corners_a, &masks.vars.corners_b]
}
pub const ENTRIES: u32 = 0x81;
const ENTRY: u32 = 0x18;
/// Each table's two steps' starts and the end they both go to.
fn steps(drops: &WindscreenDrops) -> [([usize; 2], &Dword); 2] {
    [
        ([0, 2], &drops.shape_work.work.steps[0]), // `MaskConstants::corner_angles`: dword_89C9CC, dword_89C9D4; dword_884FC4
        ([4, 6], &drops.shape_work.work.steps[1]), // dword_89C9DC, dword_89C9E4; dword_884FC8
    ]
}
const LIFT: u32 = 0xB2A7;
/// The block's lowest points (x, y).
fn offsets(drops: &WindscreenDrops, n: usize) -> [u32; 2] {
    let low = if n == 0 {
        &drops.shape_work.work.low_a
    } else {
        &drops.shape_work.work.low_b
    };
    read_first(low)
}

fn get<const N: usize>(table: &Bytes<N>, at: u32) -> Result<u32, BadInput> {
    let at = at as usize;
    if at.checked_add(4).is_none_or(|end| end > N) {
        return Err(BadInput("a read past a marks' table"));
    }
    Ok(table.get_u32(at))
}

fn put<const N: usize>(table: &Bytes<N>, at: u32, v: u32) -> Result<(), BadInput> {
    let at = at as usize;
    if at.checked_add(4).is_none_or(|end| end > N) {
        return Err(BadInput("a write past a marks' table"));
    }
    table.set_u32(at, v);
    Ok(())
}

/// The two turns step k gives table n: the second result of each of its two steps.
fn turns(drops: &WindscreenDrops, n: usize, k: u32) -> (u32, u32) {
    let (starts, end) = steps(drops)[n];
    let b: u32 = end.get();
    (
        smooth_pair(
            k,
            crate::effects::owner::effects()
                .setup
                .mask_constants
                .corner_angles[starts[0]]
                .bits(),
            b,
        )
        .1,
        smooth_pair(
            k,
            crate::effects::owner::effects()
                .setup
                .mask_constants
                .corner_angles[starts[1]]
                .bits(),
            b,
        )
        .1,
    )
}

/// `effects::make_corners` (0x4BEB25): for each table and each of 0x81 steps k, shape set 0
/// turned (`turn_shape_points`) by the second results of two eased steps (`smooth_steps`),
/// the list's farthest pair copied in as the entry's two corners, y lifted by 0xB2A7; then
/// in each entry the corners swapped (x, y) unless the second is farther from the middle
/// (`fcompp`, `g_fpuStatus`); the scales of the kind's box (its +0x24 - +0x18 over 0x100,
/// +4 - +0x1C over 0x80); each corner (x, y plus the list's lowest point, less the box's
/// +0x18, +0x1C) over them as floats; y from 0x80; and `clamp_pair`. Refuses a kind of
/// 0x22 or more, a farthest pair past 0x3F, what `turn_shape_points` refuses at any step,
/// before writing.
pub fn make_corners(masks: &Masks, drops: &WindscreenDrops) -> Result<(), Refused> {
    {
        let kind: u32 = crate::effects::owner::effects()
            .kinds
            .marks_kind
            .kind
            .bits();
        if kind >= 0x22 {
            return Err(BadInput("a marks' kind of 0x22 or more").into());
        }
        for n in 0..2 {
            let pair: [u32; 2] = read_bits(farthest_pair(drops, n));
            if pair.iter().any(|&i| i >= 0x40) {
                return Err(BadInput("a farthest pair past 0x3F").into());
            }
        }
        for n in 0..2 {
            for k in 0..ENTRIES {
                let (a, b) = turns(drops, n, k);
                turn_shape_checks(drops, a, b)?;
            }
        }
        for (n, &(starts, end)) in steps(drops).iter().enumerate() {
            for k in 0..ENTRIES {
                let first = smooth_steps(
                    drops,
                    k,
                    crate::effects::owner::effects()
                        .setup
                        .mask_constants
                        .corner_angles[starts[0]]
                        .bits(),
                    end.get(),
                )
                .1;
                let second = smooth_steps(
                    drops,
                    k,
                    crate::effects::owner::effects()
                        .setup
                        .mask_constants
                        .corner_angles[starts[1]]
                        .bits(),
                    end.get(),
                )
                .1;
                turn_shape_points(drops, first, second)?;
                let table = tables(masks)[n];
                let mut to = k * ENTRY;
                let pair: [u32; 2] = read_bits(farthest_pair(drops, n));
                let turned = turned(drops, n);
                for p in pair {
                    let from = p * 12;
                    for c in 0..3 {
                        put(table, to + 4 * c, get(turned, from + 4 * c)?)?;
                    }
                    put(table, to + 4, get(table, to + 4)?.wrapping_add(LIFT))?;
                    to += 12;
                }
            }
        }
        for k in 0..ENTRIES {
            for table in tables(masks) {
                let e = k * ENTRY;
                let c = |o: u32| -> Result<_, Refused> { Ok(get(table, e + o)? as i32) };
                let status = x87::compare_squares(c(0)?, c(4)?, c(0xC)?, c(0x10)?);
                crate::effects::owner::effects()
                    .setup
                    .work
                    .fpu_status
                    .set(status as u32);
                if status != 0 {
                    for (a, b) in [(0, 0xC), (4, 0x10)] {
                        let (x, y) = (get(table, e + a)?, get(table, e + b)?);
                        put(table, e + a, y)?;
                        put(table, e + b, x)?;
                    }
                }
            }
        }
        let bx_at = (kind * 0x30) as usize;
        let bx = |o: usize| -> Result<_, Refused> {
            Ok(masks.kind_boxes.boxes.get_u32(bx_at + o) as i32)
        };
        let divisors: [i32; 2] = crate::effects::owner::effects()
            .setup
            .mask_constants
            .divisors
            .each_ref()
            .map(|c| c.get() as i32);
        let scales = [
            x87::int_div_int(bx(0x24)?.wrapping_sub(bx(0x18)?), divisors[0]),
            x87::int_div_int(bx(4)?.wrapping_sub(bx(0x1C)?), divisors[1]),
        ];
        write_bits(&masks.vars.corner_scale, scales);
        let corners = 2 * ENTRIES;
        for k in 0..corners {
            for n in 0..2 {
                let table = tables(masks)[n];
                let e = 12 * k;
                let offset: [u32; 2] = offsets(drops, n);
                let x = get(table, e)?
                    .wrapping_add(offset[0])
                    .wrapping_sub(bx(0x18)? as u32);
                let y = get(table, e + 4)?
                    .wrapping_add(offset[1])
                    .wrapping_sub(bx(0x1C)? as u32);
                put(table, e + 4, x87::int_over(y as i32, scales[1]))?;
                put(table, e, x87::int_over(x as i32, scales[0]))?;
            }
        }
        let flips: [i32; 2] = crate::effects::owner::effects()
            .setup
            .mask_constants
            .flips
            .each_ref()
            .map(|c| c.get() as i32);
        for k in 0..corners {
            for (n, &flip) in flips.iter().enumerate() {
                let table = tables(masks)[n];
                let e = 12 * k;
                put(table, e + 4, x87::int_less(flip, get(table, e + 4)?))?;
            }
        }
        for k in 0..corners {
            for table in tables(masks) {
                let e = 12 * k;
                let out = clamp_pair([extended(get(table, e + 4)?), extended(get(table, e)?)]);
                put(table, e + 4, rounded(out[0]))?;
                put(table, e, rounded(out[1]))?;
            }
        }
    }
    Ok(())
}
