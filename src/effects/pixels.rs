//! `make_masks`' second picture: for each pixel of a shape's rows, its place against the line pair
//! (two lines) worked out, shaken by the marks' seed, and kept as four bytes in a table of 0x400 a
//! row (`MaskVars.pixel_table`).

use crate::effects::mask_store::Masks;
use crate::effects::spans::run_get;
use crate::effects::turns::{scale_rows3, set_line_turn, turn_pair, turn_pair_with};
use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;

/// The pixel table `n` (0 or 1, the shape's: `MaskPicks.pixel_table_a`, `_b`).
fn pixel_table(
    drops: &WindscreenDrops,
    n: u32,
) -> Result<&core::cell::RefCell<[u8; 0x2_0000]>, BadInput> {
    [&drops.pixel_tables.a, &drops.pixel_tables.b]
        .get(n as usize)
        .copied()
        .ok_or(BadInput("a pixel table's number that names none"))
}

/// The line of `lines` (`MaskVars.lines`) from dword `at` (0 or 6): its two ends, 3 dwords
/// apart.
fn line(lines: &[u32; 12], at: usize) -> [u32; 6] {
    let (lines, _) = lines.as_chunks::<6>();
    lines[at / 6]
}

/// The turn's rows scaled to the set lengths (`scale_rows3` on `MaskVars.turn`).
fn scale_turn(masks: &Masks, effects: &crate::effects::owner::Effects) {
    let mut turn = masks.vars.borrow().turn;
    scale_rows3(&mut turn, effects);
    masks.vars.borrow_mut().turn = turn;
}

/// One half of `map_pixel`: against the line from dword `line` of `MaskVars.lines` (its
/// other end 3 dwords on), the two points of the other line (`other`, `other` + 3), the
/// three constants, the two bytes.
struct Half {
    line: usize,
    other: usize,
    /// Which of `MaskConstants::pixel_constants` it takes: its three from there on.
    constants: usize,
    bytes: u32,
}

const HALVES: [Half; 2] = [
    Half {
        line: 0,
        other: 6,
        constants: 0,
        bytes: 2,
    },
    Half {
        line: 6,
        other: 0,
        constants: 3,
        bytes: 0,
    },
];

fn next_seed(s: u32) -> u32 {
    s.wrapping_mul(0x9D).wrapping_add(0x47)
}

/// What every pixel of a quad shares in one half of `map_pixel`: the line's turn (forward and
/// back), the line's start, the other line's ends turned into the line's frame, and the span
/// and slope between them.
pub struct HalfPlan {
    forward: [f32; 9],
    back: [f32; 9],
    start: [u32; 2],
    /// The ends in the order a1, a0, b1, b0.
    ends: [f32; 4],
    /// The span between the ends, made positive.
    span: f32,
    rise: f32,
    negative: bool,
}

/// The two halves' plans for the lines now in `MaskVars.lines`: the work `map_pixel` would
/// repeat for every pixel of a quad. Leaves the turn as the last pixel's second half does.
fn plan_halves(masks: &Masks, effects: &crate::effects::owner::Effects) -> [HalfPlan; 2] {
    HALVES.each_ref().map(|half| {
        let lines = masks.vars.borrow().lines;
        masks.vars.borrow_mut().turn = rc2k_core::math::vector::IDENTITY;
        set_line_turn(masks, line(&lines, half.line), false);
        scale_turn(masks, effects);
        let forward = masks.vars.borrow().turn;
        let (lx, ly) = (lines[half.line], lines[half.line + 1]);
        let turned = |x: [u8; 10], y: [u8; 10]| turn_pair(masks, [y, x]);
        let a = turned(
            x87::sub_80(lines[half.other], lx),
            x87::sub_80(lines[half.other + 1], ly),
        );
        let b = turned(
            x87::sub_80(lines[half.other + 3], lx),
            x87::sub_80(lines[half.other + 4], ly),
        );
        // The ends in the order a1, a0, b1, b0.
        let ends = [a[1], a[0], b[1], b[0]].map(rounded_value);
        let mut span = x87::sub(ends[2], ends[0]);
        let rise = x87::div(x87::sub(ends[3], ends[1]), span);
        let negative = span.is_sign_negative();
        if negative {
            span = -span;
        }
        set_line_turn(masks, line(&lines, half.line), true);
        scale_turn(masks, effects);
        HalfPlan {
            forward,
            back: masks.vars.borrow().turn,
            start: [lx, ly],
            ends,
            span,
            rise,
            negative,
        }
    })
}

/// `effects::map_pixel`: pixel `x`, `MaskVars.pixel[1]` into the pixel table (here its number) at
/// byte `row + 4 x`, two halves (bytes 2 and 3, then 0 and 1): the pixel turned into the line's frame
/// by the half's forward turn (`turn_pair`, 80-bit differences); the pixel's y worked out from its
/// x by a shake of the seed (one draw of 0x3FFF, a second of 0xFFFF when the x is not below the
/// span: `x87::random_step`); turned back by the half's back turn, rounded with the line's start
/// added, the low bytes kept. `plans` are the quad's (`plan_halves`). The variables the work
/// would leave (`MaskVars.turn`, `ends`, `slope`, `work_pixel`, the seed and status) are set as
/// the last half leaves them.
pub fn map_pixel(
    masks: &Masks,
    drops: &WindscreenDrops,
    row: u32,
    x: u32,
    plans: &[HalfPlan; 2],
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    let pixel_table_number = masks.vars.borrow().pixel_table;
    let table = pixel_table(drops, pixel_table_number)?;
    let pixel = {
        let mut vars = masks.vars.borrow_mut();
        vars.pixel[0] = x;
        vars.pixel
    };
    let shake = setup.work.borrow().shake;
    let mut out = [0i32; 2];
    for (half, plan) in HALVES.iter().zip(plans) {
        let [lx, ly] = plan.start;
        let p = turn_pair_with(
            &plan.forward,
            [
                x87::int_sub_80(pixel[1] as i32, ly),
                x87::int_sub_80(pixel[0] as i32, lx),
            ],
        );
        let px = rounded_value(p[1]);
        let status = x87::sub_compare(px, plan.ends[0], plan.negative, plan.span);
        setup.work.borrow_mut().fpu_status = status as u32;
        let mut seed = next_seed(setup.work.borrow().seed);
        let (mut r, mut c) = (
            seed & 0x3FFF,
            setup.mask_constants.borrow().pixel_constants[half.constants],
        );
        if status != 0x100 {
            seed = next_seed(seed);
            (r, c) = (
                seed & 0xFFFF,
                setup.mask_constants.borrow().pixel_constants[half.constants + 1],
            );
        }
        setup.work.borrow_mut().seed = seed;
        let k = setup.mask_constants.borrow().pixel_constants[half.constants + 2];
        let y = x87::random_step(
            r as i32,
            [shake, c, px, plan.ends[0], plan.rise, plan.ends[1], k],
        );
        let q = turn_pair_with(&plan.back, [extended(y), extended(px)]);
        let (qx, qy) = (rounded_value(q[1]), rounded_value(q[0]));
        out = [x87::add_to_int(qx, lx), x87::add_to_int(qy, ly)];
        let at = row.wrapping_add(x.wrapping_mul(4)).wrapping_add(half.bytes);
        let at = at as usize;
        table
            .borrow_mut()
            .get_mut(at..at + 2)
            .ok_or(BadInput("a mask pixel past the end of its table"))?
            .copy_from_slice(&[out[0] as u8, out[1] as u8]);
    }
    let mut vars = masks.vars.borrow_mut();
    vars.work_pixel = out.map(|v| v as u32);
    if let Some(last) = plans.last() {
        vars.turn = last.back;
        for (n, end) in last.ends.iter().enumerate() {
            vars.ends[[0, 1, 3, 4][n]] = *end;
        }
        vars.slope = [last.span, last.rise];
    }
    Ok(())
}

/// The rows a shape entry may fill: x and length inside 0x100, rows inside 0x80.
fn rows_fit(masks: &Masks, entries: u32, index: u32) -> Result<(), Refused> {
    {
        let e = entries.wrapping_add(index.wrapping_mul(16));
        let (spans, row, count) = (
            run_get(masks, e)?,
            run_get(masks, e + 4)? as i32,
            run_get(masks, e + 8)? as i32,
        );
        if count > 0 && (row < 0 || row + count > 0x80) {
            return Err(BadInput("pixel rows outside the table").into());
        }
        for i in 0..count.max(0) as u32 {
            let (x, len) = (
                crate::effects::masks::work_get(masks, spans.wrapping_add(8 * i))? as i32,
                crate::effects::masks::work_get(masks, spans.wrapping_add(8 * i + 4))? as i32,
            );
            if len > 0 && (x < 0 || x + len > 0x100) {
                return Err(BadInput("pixel spans outside the table").into());
            }
        }
    }
    Ok(())
}

/// `effects::map_shape_rows`: entry `index` of the list at `entries`: for each of its rows
/// (`MaskVars.pixel[1]` the row, from the table's row in `MaskVars.pixel_table`, here its byte
/// offset in the table) `MaskVars.row_distance` the row's squared distance
/// (`x87::int_plus_less_squared`) and `map_pixel` for each of its span's pixels. Refuses rows or
/// spans outside the 0x100 x 0x80 table, before writing.
pub(super) fn map_shape_rows(
    masks: &Masks,
    drops: &WindscreenDrops,
    entries: u32,
    index: u32,
    plans: &mut Option<[HalfPlan; 2]>,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        rows_fit(masks, entries, index)?;
        let e = entries.wrapping_add(index.wrapping_mul(16));
        let (spans, row, count) = (
            run_get(masks, e)?,
            run_get(masks, e + 4)?,
            run_get(masks, e + 8)?,
        );
        masks.vars.borrow_mut().pixel[1] = row;
        let mut at = row.wrapping_mul(0x400);
        for i in 0..(count as i32).max(0) as u32 {
            let (x, len) = (
                crate::effects::masks::work_get(masks, spans.wrapping_add(8 * i))?,
                crate::effects::masks::work_get(masks, spans.wrapping_add(8 * i + 4))?,
            );
            let (y, middle_y) = {
                let vars = masks.vars.borrow();
                (vars.pixel[1], vars.middle[1])
            };
            let row_add = effects.setup.mask_constants.borrow().row_add;
            masks.vars.borrow_mut().row_distance =
                x87::int_plus_less_squared(y as i32, row_add, middle_y);
            for n in 0..(len as i32).max(0) as u32 {
                let plans = plans.get_or_insert_with(|| plan_halves(masks, effects));
                map_pixel(masks, drops, at, x.wrapping_add(n), plans, effects)?;
            }
            masks.vars.borrow_mut().pixel[1] = y.wrapping_add(1);
            at = at.wrapping_add(0x400);
        }
    }
    Ok(())
}

/// `effects::map_shape`: quad `shape` of the shape `shapes`:
/// `MaskVars.mask_value` the quad, its two entries (2 n, 2 n + 1) of the shape's rows mapped
fn map_shape(
    masks: &Masks,
    drops: &WindscreenDrops,
    entries: u32,
    shape: u32,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let first = shape.wrapping_add(shape);
        rows_fit(masks, entries, first)?;
        rows_fit(masks, entries, first.wrapping_add(1))?;
        masks.vars.borrow_mut().mask_value = shape;
        // The quad's plans are made at its first pixel, for both its entries.
        let mut plans = None;
        map_shape_rows(masks, drops, entries, first, &mut plans, effects)?;
        map_shape_rows(
            masks,
            drops,
            entries,
            first.wrapping_add(1),
            &mut plans,
            effects,
        )?;
    }
    Ok(())
}

/// Where each line-pair float comes from in a corner entry pair.
const LINE_FROM: [(u32, u32); 8] = [
    (0, 0),
    (4, 4),
    (0xC, 0xC),
    (0x10, 0x10),
    (0x18, 0x18),
    (0x1C, 0x1C),
    (0x24, 0x24),
    (0x28, 0x28),
];

/// `effects::map_shapes`: with the x87 rounding toward zero (its word put back after),
/// shape `n`'s pixel table (0x8000 dwords) cleared and made the table in use, its corner
/// table kept, the shape's middle (`middle`) kept; for each of its quads the line pair
/// from corner entries k and k + 1 into `MaskVars.lines` and `map_shape`. The kept tables
/// (`MaskPicks.table_in_use`, `MaskPicks.corners`) hold the shape's number. Refuses a
/// count of 0 or less or past 0x80, and rows or spans outside the table, before writing.
pub(super) fn map_shapes(
    masks: &Masks,
    drops: &WindscreenDrops,
    n: u32,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let shape = *masks
            .vars
            .borrow()
            .shapes
            .get(n as usize)
            .ok_or(BadInput("a shape's number that names none"))?;
        let table = pixel_table(drops, n)?;
        let count = shape.count as i32;
        if count <= 0 || count > 0x80 {
            return Err(BadInput("no shapes to map, or more than 0x80").into());
        }
        let entries = shape.rows;
        for k in 0..2 * count as u32 {
            rows_fit(masks, entries, k)?;
        }
        let came = x87::control_word();
        x87::set_control_word((came & 0xF3FF) | 0xC00);
        drops.picks.borrow_mut().shapes = n;
        drops.picks.borrow_mut().table_in_use = n;
        drops.picks.borrow_mut().corners = n;
        table.borrow_mut().fill(0);
        {
            let (table_in_use, corner_table) = {
                let picks = drops.picks.borrow();
                (picks.table_in_use, picks.corners)
            };
            let mut vars = masks.vars.borrow_mut();
            vars.pixel_table = table_in_use;
            vars.pixel_corners = corner_table;
            vars.middle = [x87::loaded(shape.middle[0]), x87::loaded(shape.middle[1])];
        }
        for k in 0..count as u32 {
            let e = k * 0x18;
            for (to, from) in LINE_FROM {
                let word = {
                    let vars = masks.vars.borrow();
                    let at = (e + from) as usize;
                    *vars.corners[n as usize]
                        .get(at..)
                        .and_then(|rest| rest.first_chunk::<4>())
                        .ok_or(BadInput("a corner entry"))?
                };
                masks.vars.borrow_mut().lines[to as usize / 4] = u32::from_le_bytes(word);
            }
            map_shape(masks, drops, entries, k, effects)?;
        }
        x87::set_control_word(came);
    }
    Ok(())
}
