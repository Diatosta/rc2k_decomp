//! `make_masks`' corner tables (0x81 entries of two corners): shape set 0 turned at 0x81
//! eased steps, each list's farthest pair taken as an entry's two corners, ordered, scaled
//! into the kind's box, flipped and held (`clamp_pair`); `span_ring` makes the rings of
//! them.

use crate::effects::mask_store::Masks;
use crate::effects::shape_points::{farthest_pair, turn_shape_checks, turn_shape_points};
use crate::effects::turns::{clamp_pair, smooth_pair, smooth_steps};
use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::{extended, rounded};
use crate::math::x87;

pub const ENTRIES: u32 = 0x81;
const ENTRY: u32 = 0x18;
/// Each table's two steps' starts and the end they both go to.
fn steps(drops: &WindscreenDrops) -> [([usize; 2], u32); 2] {
    let work = drops.shape_work.borrow();
    [
        ([0, 2], work.work.steps[0]), // `MaskConstants::corner_angles`: MaskConstants.corner_angles[0], MaskConstants.corner_angles[2]; ShapeWork.work.steps[0]
        ([4, 6], work.work.steps[1]), // MaskConstants.corner_angles[4], MaskConstants.corner_angles[6]; ShapeWork.work.steps[1]
    ]
}
const LIFT: u32 = 0xB2A7;
/// The block's lowest points (x, y).
fn offsets(drops: &WindscreenDrops, n: usize) -> [u32; 2] {
    let low = drops.shape_work.borrow().work.low[n];
    [low[0], low[1]]
}

fn get<const N: usize>(table: &[u8; N], at: u32) -> Result<u32, BadInput> {
    let at = at as usize;
    if at.checked_add(4).is_none_or(|end| end > N) {
        return Err(BadInput("a read past a marks' table"));
    }
    Ok(u32::from_le_bytes([
        table[at],
        table[at + 1],
        table[at + 2],
        table[at + 3],
    ]))
}

fn put<const N: usize>(table: &mut [u8; N], at: u32, v: u32) -> Result<(), BadInput> {
    let at = at as usize;
    if at.checked_add(4).is_none_or(|end| end > N) {
        return Err(BadInput("a write past a marks' table"));
    }
    table[at..at + 4].copy_from_slice(&v.to_le_bytes());
    Ok(())
}

/// The two turns step k gives table n: the second result of each of its two steps.
fn turns(
    drops: &WindscreenDrops,
    effects: &crate::effects::owner::Effects,
    n: usize,
    k: u32,
) -> (u32, u32) {
    let setup = &effects.setup;
    let (starts, end) = steps(drops)[n];
    let b: u32 = end;
    (
        smooth_pair(
            k,
            setup.mask_constants.borrow().corner_angles[starts[0]],
            b,
            effects,
        )
        .1,
        smooth_pair(
            k,
            setup.mask_constants.borrow().corner_angles[starts[1]],
            b,
            effects,
        )
        .1,
    )
}

/// `effects::make_corners`: for each table and each of 0x81 steps k, shape set 0 turned
/// (`turn_shape_points`) by the second results of two eased steps (`smooth_steps`), the
/// list's farthest pair copied in as the entry's two corners, y lifted by 0xB2A7; then in
/// each entry the corners swapped (x, y) unless the second is farther from the middle; the
/// scales of the kind's box (the difference of two of its words over 0x100, of two others over
/// 0x80); each corner (x, y plus the list's lowest point, less the box's two minimum words) over
/// them as
/// floats; y from 0x80; and `clamp_pair`. Refuses a kind of 0x22 or more, a farthest pair
/// past 0x3F, what `turn_shape_points` refuses at any step, before writing.
pub(super) fn make_corners(
    masks: &Masks,
    drops: &WindscreenDrops,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let kind: u32 = effects.kinds.marks_kind.get();
        if kind >= 0x22 {
            return Err(BadInput("a marks' kind of 0x22 or more").into());
        }
        for n in 0..2 {
            let pair: [u32; 2] = farthest_pair(drops, n);
            if pair.iter().any(|&i| i >= 0x40) {
                return Err(BadInput("a farthest pair past 0x3F").into());
            }
        }
        for n in 0..2 {
            for k in 0..ENTRIES {
                let (a, b) = turns(drops, effects, n, k);
                turn_shape_checks(drops, a, b)?;
            }
        }
        for (n, &(starts, end)) in steps(drops).iter().enumerate() {
            for k in 0..ENTRIES {
                let first = smooth_steps(
                    k,
                    effects.setup.mask_constants.borrow().corner_angles[starts[0]],
                    end,
                    effects,
                )
                .1;
                let second = smooth_steps(
                    k,
                    effects.setup.mask_constants.borrow().corner_angles[starts[1]],
                    end,
                    effects,
                )
                .1;
                turn_shape_points(&effects.stage()?.object_points, drops, first, second)?;
                let mut to = k * ENTRY;
                let pair: [u32; 2] = farthest_pair(drops, n);
                for p in pair {
                    let from = p * 12;
                    for c in 0..3 {
                        let word = get(&drops.shape_work.borrow().work.turned[n], from + 4 * c)?;
                        put(&mut masks.vars.borrow_mut().corners[n], to + 4 * c, word)?;
                    }
                    let mut vars = masks.vars.borrow_mut();
                    let table = &mut vars.corners[n];
                    put(table, to + 4, get(table, to + 4)?.wrapping_add(LIFT))?;
                    to += 12;
                }
            }
        }
        for k in 0..ENTRIES {
            for n in 0..2 {
                let e = k * ENTRY;
                let mut vars = masks.vars.borrow_mut();
                let table = &mut vars.corners[n];
                let c = |table: &[u8; 3096], o: u32| -> Result<_, Refused> {
                    Ok(get(table, e + o)? as i32)
                };
                let status = x87::compare_squares(
                    c(table, 0)?,
                    c(table, 4)?,
                    c(table, 0xC)?,
                    c(table, 0x10)?,
                );
                effects.setup.work.borrow_mut().fpu_status = status as u32;
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
        let divisors: [i32; 2] = effects
            .setup
            .mask_constants
            .borrow()
            .divisors
            .map(|c| c as i32);
        let scales = [
            x87::int_ratio(bx(0x24)?.wrapping_sub(bx(0x18)?), divisors[0]),
            x87::int_ratio(bx(4)?.wrapping_sub(bx(0x1C)?), divisors[1]),
        ];
        masks.vars.borrow_mut().corner_scale = scales;
        let corners = 2 * ENTRIES;
        for k in 0..corners {
            for n in 0..2 {
                let mut vars = masks.vars.borrow_mut();
                let table = &mut vars.corners[n];
                let e = 12 * k;
                let offset: [u32; 2] = offsets(drops, n);
                let x = get(table, e)?
                    .wrapping_add(offset[0])
                    .wrapping_sub(bx(0x18)? as u32);
                let y = get(table, e + 4)?
                    .wrapping_add(offset[1])
                    .wrapping_sub(bx(0x1C)? as u32);
                put(table, e + 4, x87::int_over(y as i32, scales[1]).to_bits())?;
                put(table, e, x87::int_over(x as i32, scales[0]).to_bits())?;
            }
        }
        let flips: [i32; 2] = effects
            .setup
            .mask_constants
            .borrow()
            .flips
            .map(|c| c as i32);
        for k in 0..corners {
            for (n, &flip) in flips.iter().enumerate() {
                let mut vars = masks.vars.borrow_mut();
                let table = &mut vars.corners[n];
                let e = 12 * k;
                put(
                    table,
                    e + 4,
                    x87::int_less(flip, get(table, e + 4)?).to_bits(),
                )?;
            }
        }
        for k in 0..corners {
            for n in 0..2 {
                let mut vars = masks.vars.borrow_mut();
                let table = &mut vars.corners[n];
                let e = 12 * k;
                let out = clamp_pair(
                    [extended(get(table, e + 4)?), extended(get(table, e)?)],
                    effects,
                );
                put(table, e + 4, rounded(out[0]))?;
                put(table, e, rounded(out[1]))?;
            }
        }
    }
    Ok(())
}
