//! `make_masks`' second picture: for each pixel of a shape's rows, its place against the line pair
//! (two lines) worked out, shaken by the marks' seed, and kept as four bytes in a table of 0x400 a
//! row (`MaskVars.pixel_table`).

use crate::effects::mask_store::Masks;
use crate::effects::mask_vars::Shape;
use crate::effects::spans::run_get;
use crate::effects::turns::{scale_rows3, set_line_turn, turn_pair, turn_pair_with};
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;
use crate::memory::session::{PlainBytes, PlainDword, PlainFloat, read_bits, write_bits};

/// The pixel table `n` (0 or 1, the shape's: `MaskPicks.pixel_table_a`, `_b`).
fn pixel_table(drops: &WindscreenDrops, n: u32) -> Result<&PlainBytes<0x2_0000>, BadInput> {
    [&drops.picks.pixel_table_a, &drops.picks.pixel_table_b]
        .get(n as usize)
        .copied()
        .ok_or(BadInput("a pixel table's number that names none"))
}

/// The line of `MaskVars.lines` from dword `at` (0 or 6): its two ends, 3 dwords apart.
fn line(masks: &Masks, at: usize) -> &[PlainDword; 6] {
    let (lines, _) = masks.vars.lines.as_chunks::<6>();
    &lines[at / 6]
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
    forward: [PlainFloat; 9],
    back: [PlainFloat; 9],
    start: [u32; 2],
    /// The ends in the order a1, a0, b1, b0.
    ends: [f32; 4],
    /// The span between the ends, made positive.
    span: f32,
    rise: f32,
    negative: bool,
}

/// A copy of the turn's nine floats.
fn turn_copy(masks: &Masks) -> [PlainFloat; 9] {
    std::array::from_fn(|n| PlainFloat::from_bits(masks.vars.turn[n].bits()))
}

/// The two halves' plans for the lines now in `MaskVars.lines`: the work `map_pixel` would
/// repeat for every pixel of a quad. Leaves the turn as the last pixel's second half does.
fn plan_halves(masks: &Masks, effects: &crate::effects::owner::Effects) -> [HalfPlan; 2] {
    HALVES.each_ref().map(|half| {
        let lines = &masks.vars.lines;
        crate::math::matrix3::identity_matrix3(&masks.vars.turn);
        set_line_turn(masks, line(masks, half.line), false);
        scale_rows3(&masks.vars.turn, effects);
        let forward = turn_copy(masks);
        let (lx, ly) = (lines[half.line].get(), lines[half.line + 1].get());
        let turned = |x: [u8; 10], y: [u8; 10]| turn_pair(masks, [y, x]);
        let a = turned(
            x87::sub_80(lines[half.other].get(), lx),
            x87::sub_80(lines[half.other + 1].get(), ly),
        );
        let b = turned(
            x87::sub_80(lines[half.other + 3].get(), lx),
            x87::sub_80(lines[half.other + 4].get(), ly),
        );
        // The ends in the order a1, a0, b1, b0.
        let ends = [a[1], a[0], b[1], b[0]].map(rounded_value);
        let mut span = x87::sub(ends[2], ends[0]);
        let rise = x87::div(x87::sub(ends[3], ends[1]), span);
        let negative = span.is_sign_negative();
        if negative {
            span = -span;
        }
        set_line_turn(masks, line(masks, half.line), true);
        scale_rows3(&masks.vars.turn, effects);
        HalfPlan {
            forward,
            back: turn_copy(masks),
            start: [lx, ly],
            ends,
            span,
            rise,
            negative,
        }
    })
}

/// `effects::map_pixel`: pixel `x`, `MaskVars.pixel[1]` into the pixel table (here its number) at
/// byte `row + 4 x`, two halves (bytes +2, +3, then +0, +1): the pixel turned into the line's frame
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
    let table = pixel_table(drops, masks.vars.pixel_table.get())?;
    masks.vars.pixel[0].set(x);
    let pixel: [u32; 2] = read_bits(&masks.vars.pixel);
    let shake = setup.work.shake.get();
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
        setup.work.fpu_status.set(status as u32);
        let mut seed = next_seed(setup.work.seed.get());
        let (mut r, mut c) = (
            seed & 0x3FFF,
            setup.mask_constants.pixel_constants[half.constants].get(),
        );
        if status != 0x100 {
            seed = next_seed(seed);
            (r, c) = (
                seed & 0xFFFF,
                setup.mask_constants.pixel_constants[half.constants + 1].get(),
            );
        }
        setup.work.seed.set(seed);
        let k = setup.mask_constants.pixel_constants[half.constants + 2].get();
        let y = x87::random_step(
            r as i32,
            [shake, c, px, plan.ends[0], plan.rise, plan.ends[1], k],
        );
        let q = turn_pair_with(&plan.back, [extended(y), extended(px)]);
        let (qx, qy) = (rounded_value(q[1]), rounded_value(q[0]));
        out = [x87::add_to_int(qx, lx), x87::add_to_int(qy, ly)];
        let at = row.wrapping_add(x.wrapping_mul(4)).wrapping_add(half.bytes);
        table.write_at(at as usize, &[out[0] as u8, out[1] as u8], "a mask pixel")?;
    }
    write_bits(&masks.vars.work_pixel, out.map(|v| v as u32));
    if let Some(last) = plans.last() {
        for (cell, from) in masks.vars.turn.iter().zip(&last.back) {
            cell.set_bits(from.bits());
        }
        for (n, end) in last.ends.iter().enumerate() {
            masks.vars.ends[[0, 1, 3, 4][n]].set(*end);
        }
        masks.vars.slope[0].set(last.span);
        masks.vars.slope[1].set(last.rise);
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
pub fn map_shape_rows(
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
        masks.vars.pixel[1].set(row);
        let mut at = row.wrapping_mul(0x400);
        for i in 0..(count as i32).max(0) as u32 {
            let (x, len) = (
                crate::effects::masks::work_get(masks, spans.wrapping_add(8 * i))?,
                crate::effects::masks::work_get(masks, spans.wrapping_add(8 * i + 4))?,
            );
            let y: u32 = masks.vars.pixel[1].get();
            masks.vars.row_distance.set(x87::int_plus_less_squared(
                y as i32,
                &effects.setup.mask_constants.row_add,
                &masks.vars.middle[1],
            ));
            for n in 0..(len as i32).max(0) as u32 {
                let plans = plans.get_or_insert_with(|| plan_halves(masks, effects));
                map_pixel(masks, drops, at, x.wrapping_add(n), plans, effects)?;
            }
            masks.vars.pixel[1].set(y.wrapping_add(1));
            at = at.wrapping_add(0x400);
        }
    }
    Ok(())
}

/// `effects::map_shape`: quad `shape` of the shape `shapes`:
/// `MaskVars.mask_value` the quad, its two entries (2 n, 2 n + 1) of the rows at +8 mapped
pub fn map_shape(
    masks: &Masks,
    drops: &WindscreenDrops,
    shapes: &Shape,
    shape: u32,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let entries = shapes.rows.get();
        let first = shape.wrapping_add(shape);
        rows_fit(masks, entries, first)?;
        rows_fit(masks, entries, first.wrapping_add(1))?;
        masks.vars.mask_value.set(shape);
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
/// table kept, the shape's middle (+0xC, +0x10) kept; for each of its +4 quads the line pair
/// from corner entries k and k + 1 into `MaskVars.lines` and `map_shape`. The kept tables
/// (`MaskPicks.table_in_use`, `MaskPicks.corners`) hold the shape's number. Refuses a
/// count of 0 or less or past 0x80, and rows or spans outside the table, before writing.
pub fn map_shapes(
    masks: &Masks,
    drops: &WindscreenDrops,
    n: u32,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let shapes = masks
            .vars
            .shapes
            .get(n as usize)
            .ok_or(BadInput("a shape's number that names none"))?;
        let table = pixel_table(drops, n)?;
        let corners = crate::effects::corners::tables(masks)[n as usize];
        let count = shapes.count.get() as i32;
        if count <= 0 || count > 0x80 {
            return Err(BadInput("no shapes to map, or more than 0x80").into());
        }
        let entries = shapes.rows.get();
        for k in 0..2 * count as u32 {
            rows_fit(masks, entries, k)?;
        }
        let came = x87::control_word();
        x87::set_control_word((came & 0xF3FF) | 0xC00);
        drops.picks.shapes.set(n);
        drops.picks.table_in_use.set(n);
        drops.picks.corners.set(n);
        table.write_at(0, &vec![0; 0x2_0000], "the pixel table")?;
        masks.vars.pixel_table.set(drops.picks.table_in_use.get());
        masks.vars.pixel_corners.set(drops.picks.corners.get());
        write_bits(
            &masks.vars.middle,
            [
                x87::loaded(&shapes.middle[0]).to_bits(),
                x87::loaded(&shapes.middle[1]).to_bits(),
            ],
        );
        for k in 0..count as u32 {
            let e = k * 0x18;
            for (to, from) in LINE_FROM {
                let bytes = corners.read_at((e + from) as usize, 4, "a corner entry")?;
                masks.vars.lines[to as usize / 4]
                    .set(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
            }
            map_shape(masks, drops, shapes, k, effects)?;
        }
        x87::set_control_word(came);
    }
    Ok(())
}
