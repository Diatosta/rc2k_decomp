//! `make_masks`' second picture (IDA, 2026-10-17): for each pixel of a shape's rows, its
//! place against the line pair at 0x82E3DA (two lines: 0x82E3DA -> 0x82E3E6 and 0x82E3F2
//! -> 0x82E3FE) worked out, shaken by the marks' seed, and kept as four bytes in a table
//! of 0x400 a row (`dword_8445D2`).

use crate::effects::mask_constants::MASK_CONSTANTS;
use crate::effects::mask_picks::PICKS;
use crate::effects::mask_vars::MASKS;
use crate::effects::mask_vars::Shape;
use crate::effects::spans::run_get;
use crate::effects::turns::{scale_rows3, set_line_turn, turn_pair};
use crate::effects::work_store::MARKS_WORK;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::session::{Bytes, Dword, read_bits, write_bits};

/// The pixel table `n` (0 or 1, the shape's: `PICKS.pixel_table_a`, `_b`).
fn pixel_table(n: u32) -> Result<&'static Bytes<0x2_0000>, BadInput> {
    [&PICKS.pixel_table_a, &PICKS.pixel_table_b]
        .get(n as usize)
        .copied()
        .ok_or(BadInput("a pixel table's number that names none"))
}

/// The line of `g_marksLines` from dword `at` (0 or 6): its two ends, 3 dwords apart.
fn line(at: usize) -> &'static [Dword; 6] {
    let (lines, _) = MASKS.lines.as_chunks::<6>();
    &lines[at / 6]
}

/// One half of `map_pixel`: against the line from dword `line` of `g_marksLines` (its
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

/// `effects::map_pixel` (0x4BEFD2): pixel `x`, `dword_833DB6` into the pixel table
/// (`g_marksPixelTable`, here its number) at byte `row + 4 x`, two
/// halves (bytes +2, +3, then +0, +1): the turn of the half's line (identity, then
/// `set_line_turn`, `scale_rows3`); the other line's two ends and the pixel turned into
/// its frame (`turn_pair`, 80-bit differences); the slope between the ends; the pixel's y
/// worked out from its x by a shake of the seed (one draw of 0x3FFF, a second of 0xFFFF
/// when the x is not below the span: `x87::random_step`); turned back
/// (`set_line_turn_back`), rounded with the line's start added, the low bytes kept.
pub fn map_pixel(row: u32, x: u32) -> Result<(), Refused> {
    let table = pixel_table(MASKS.pixel_table.get())?;
    let f = |o: usize| -> Result<_, Refused> { Ok(MASKS.lines[o].get()) };
    MASKS.pixel[0].set(x);
    let pixel: [u32; 2] = read_bits(&MASKS.pixel);
    let shake: u32 = MARKS_WORK.shake.get();
    for half in HALVES {
        write_bits(&MASKS.work_pixel, pixel);
        crate::scene::matrix3::identity_matrix3(&MASKS.turn);
        set_line_turn(line(half.line), false);
        scale_rows3(&MASKS.turn);
        let (lx, ly) = (f(half.line)?, f(half.line + 1)?);
        let turned = |x: [u8; 10], y: [u8; 10]| turn_pair([y, x]);
        let a = turned(
            x87::sub_80(f(half.other)?, lx),
            x87::sub_80(f(half.other + 1)?, ly),
        );
        let b = turned(
            x87::sub_80(f(half.other + 3)?, lx),
            x87::sub_80(f(half.other + 4)?, ly),
        );
        let p = turned(
            x87::int_sub_80(pixel[0] as i32, lx),
            x87::int_sub_80(pixel[1] as i32, ly),
        );
        // 0x82E40A, _40E, then _416, _41A (0x82E412 is left).
        let ends = [rounded(a[1]), rounded(a[0]), rounded(b[1]), rounded(b[0])];
        MASKS.ends[0].set_bits(ends[0]);
        MASKS.ends[1].set_bits(ends[1]);
        MASKS.ends[3].set_bits(ends[2]);
        MASKS.ends[4].set_bits(ends[3]);
        let (px, py) = (rounded(p[1]), rounded(p[0]));
        write_bits(&MASKS.work_pixel, [px, py]);
        let mut span = x87::sub(ends[2], ends[0]);
        let rise = x87::div(x87::sub(ends[3], ends[1]), span);
        write_bits(&MASKS.slope, [span, rise]);
        let negative = span & 0x8000_0000 != 0;
        if negative {
            span ^= 0x8000_0000;
            MASKS.slope[0].set_bits(span);
        }
        let status = x87::sub_compare(px, ends[0], negative, span);
        MARKS_WORK.fpu_status.set(status as u32);
        let mut seed = next_seed(MARKS_WORK.seed.get());
        let (mut r, mut c) = (
            seed & 0x3FFF,
            MASK_CONSTANTS.pixel_constants[half.constants].bits(),
        );
        if status != 0x100 {
            seed = next_seed(seed);
            (r, c) = (
                seed & 0xFFFF,
                MASK_CONSTANTS.pixel_constants[half.constants + 1].bits(),
            );
        }
        MARKS_WORK.seed.set(seed);
        let k: u32 = MASK_CONSTANTS.pixel_constants[half.constants + 2].bits();
        let y = x87::random_step(r as i32, [shake, c, px, ends[0], rise, ends[1], k]);
        MASKS.work_pixel[1].set(y);
        set_line_turn(line(half.line), true);
        scale_rows3(&MASKS.turn);
        let q = turn_pair([extended(y), extended(px)]);
        let (qx, qy) = (rounded(q[1]), rounded(q[0]));
        let out = [x87::add_to_int(qx, lx), x87::add_to_int(qy, ly)];
        write_bits(&MASKS.work_pixel, out.map(|v| v as u32));
        let at = row.wrapping_add(x.wrapping_mul(4)).wrapping_add(half.bytes);
        table.write_at(at as usize, &[out[0] as u8, out[1] as u8], "a mask pixel")?;
    }
    Ok(())
}

/// The rows a shape entry may fill: x and length inside 0x100, rows inside 0x80.
fn rows_fit(entries: u32, index: u32) -> Result<(), Refused> {
    {
        let e = entries.wrapping_add(index.wrapping_mul(16));
        let (spans, row, count) = (run_get(e)?, run_get(e + 4)? as i32, run_get(e + 8)? as i32);
        if count > 0 && (row < 0 || row + count > 0x80) {
            return Err(BadInput("pixel rows outside the table").into());
        }
        for i in 0..count.max(0) as u32 {
            let (x, len) = (
                crate::effects::masks::work_get(spans.wrapping_add(8 * i))? as i32,
                crate::effects::masks::work_get(spans.wrapping_add(8 * i + 4))? as i32,
            );
            if len > 0 && (x < 0 || x + len > 0x100) {
                return Err(BadInput("pixel spans outside the table").into());
            }
        }
    }
    Ok(())
}

/// `effects::map_shape_rows` (0x4BE855): entry `index` of the list at `entries`: for each
/// of its rows (`dword_833DB6` the row, from the table's row in `dword_8445D2`, here its
/// byte offset in the table)
/// `flt_8445E2` the row's squared distance (`x87::int_plus_less_squared`) and `map_pixel`
/// for each of its span's pixels. Refuses rows or spans outside the 0x100 x 0x80 table,
/// before writing.
pub fn map_shape_rows(entries: u32, index: u32) -> Result<(), Refused> {
    {
        rows_fit(entries, index)?;
        let e = entries.wrapping_add(index.wrapping_mul(16));
        let (spans, row, count) = (run_get(e)?, run_get(e + 4)?, run_get(e + 8)?);
        MASKS.pixel[1].set(row);
        let mut at = row.wrapping_mul(0x400);
        for i in 0..(count as i32).max(0) as u32 {
            let (x, len) = (
                crate::effects::masks::work_get(spans.wrapping_add(8 * i))?,
                crate::effects::masks::work_get(spans.wrapping_add(8 * i + 4))?,
            );
            let y: u32 = MASKS.pixel[1].get();
            MASKS.row_distance.set_bits(x87::int_plus_less_squared(
                y as i32,
                MASK_CONSTANTS.row_add.bits(),
                MASKS.middle[1].bits(),
            ));
            for n in 0..(len as i32).max(0) as u32 {
                map_pixel(at, x.wrapping_add(n))?;
            }
            MASKS.pixel[1].set(y.wrapping_add(1));
            at = at.wrapping_add(0x400);
        }
    }
    Ok(())
}

/// `effects::map_shape` (0x4BE83E): quad `shape` of the shape `shapes`:
/// `g_marksMaskValue` the quad, its two entries (2 n, 2 n + 1) of the rows at +8 mapped
/// (`map_shape_rows`). Refuses as `map_shape_rows`, before writing.
pub fn map_shape(shapes: &Shape, shape: u32) -> Result<(), Refused> {
    {
        let entries = shapes.rows.get();
        let first = shape.wrapping_add(shape);
        rows_fit(entries, first)?;
        rows_fit(entries, first.wrapping_add(1))?;
        MASKS.mask_value.set(shape);
        map_shape_rows(entries, first)?;
        map_shape_rows(entries, first.wrapping_add(1))?;
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

/// `effects::map_shapes` (0x4BE752): with the x87 rounding toward zero (its word put back
/// after), shape `n`'s pixel table (0x8000 dwords) cleared and made `g_marksPixelTable`,
/// its corner table kept, the shape's middle (+0xC, +0x10) into 0x8445D6; for each of its
/// +4 quads the line pair from corner entries k and k + 1 into `g_marksLines` and
/// `map_shape`. The kept tables (`dword_844E18`, `dword_844E1C`, `g_marksPixelTable`,
/// `dword_8445DE`) hold the shape's number where the original kept addresses. Refuses a
/// count of 0 or less or past 0x80 (its loop tests after counting), and rows or spans
/// outside the table, before writing.
pub fn map_shapes(n: u32) -> Result<(), Refused> {
    {
        let shapes = MASKS
            .shapes
            .get(n as usize)
            .ok_or(BadInput("a shape's number that names none"))?;
        let table = pixel_table(n)?;
        let corners = crate::effects::corners::tables()[n as usize];
        let count = shapes.count.get() as i32;
        if count <= 0 || count > 0x80 {
            return Err(BadInput("no shapes to map, or more than 0x80").into());
        }
        let entries = shapes.rows.get();
        for k in 0..2 * count as u32 {
            rows_fit(entries, k)?;
        }
        let came = x87::control_word();
        x87::set_control_word((came & 0xF3FF) | 0xC00);
        PICKS.shapes.set(n);
        PICKS.table_in_use.set(n);
        PICKS.corners.set(n);
        table.write_at(0, &vec![0; 0x2_0000], "the pixel table")?;
        MASKS.pixel_table.set(PICKS.table_in_use.get());
        MASKS.pixel_corners.set(PICKS.corners.get());
        write_bits(
            &MASKS.middle,
            [
                x87::loaded(shapes.middle[0].bits()),
                x87::loaded(shapes.middle[1].bits()),
            ],
        );
        for k in 0..count as u32 {
            let e = k * 0x18;
            for (to, from) in LINE_FROM {
                let bytes = corners.read_at((e + from) as usize, 4, "a corner entry")?;
                MASKS.lines[to as usize / 4]
                    .set(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
            }
            map_shape(shapes, k)?;
        }
        x87::set_control_word(came);
    }
    Ok(())
}
