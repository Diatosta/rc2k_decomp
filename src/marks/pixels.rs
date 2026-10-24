//! `make_masks`' second picture (IDA, 2026-10-17): for each pixel of a shape's rows, its
//! place against the line pair at 0x82E3DA (two lines: 0x82E3DA -> 0x82E3E6 and 0x82E3F2
//! -> 0x82E3FE) worked out, shaken by the marks' seed, and kept as four bytes in a table
//! of 0x400 a row (`dword_8445D2`).

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::setup::SEED_RVA;
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::marks::turns::{TURN_RVA, scale_rows3, set_line_turn, turn_pair};
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::vector3::STATUS_RVA;

pub const PIXEL_RVA: Slot = Slot::at(0x833DB2); // g_marksPixel: x, then y (dword_833DB6)
pub const WORK_RVA: Slot = Slot::at(0x82E3D2); // dword_82E3D2, dword_82E3D6: the pixel worked on
const SLOPE_RVA: Slot = Slot::at(0x82E3C6); // flt_82E3C6, flt_82E3CA
const ENDS_RVA: Slot = Slot::at(0x82E40A); // flt_82E40A .. flt_82E41A: the line's ends turned
const LINES_RVA: Slot = Slot::at(0x82E3DA); // g_marksLines: 0x3DA, 0x3DE .. 0x3FE, 0x402
const SHAKE_RVA: usize = rva(0x71C870); // flt_71C870, 1 / 65536
pub const TABLE_RVA: Slot = Slot::at(0x8445D2); // g_marksPixelTable: rows of 0x400
const ROW_RVA: Slot = Slot::at(0x8445E2); // flt_8445E2
const ROW_ADD_RVA: Slot = Slot::at(0x89C9BC); // flt_89C9BC
const ROW_LESS_RVA: Slot = Slot::at(0x8445DA); // flt_8445DA

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// One half of `map_pixel`: against the line at `line` (its other end 0xC on), the two
/// points of the other line (`other`, `other` + 0xC), the three constants, the two bytes.
struct Half {
    line: u32,
    other: u32,
    constants: [Slot; 3],
    bytes: u32,
}

const HALVES: [Half; 2] = [
    Half {
        line: 0,
        other: 0x18,
        constants: [Slot::at(0x89CA0C), Slot::at(0x89CA10), Slot::at(0x89CA14)],
        bytes: 2,
    },
    Half {
        line: 0x18,
        other: 0,
        constants: [Slot::at(0x89CA18), Slot::at(0x89CA1C), Slot::at(0x89CA20)],
        bytes: 0,
    },
];

fn next_seed(s: u32) -> u32 {
    s.wrapping_mul(0x9D).wrapping_add(0x47)
}

/// `marks::map_pixel` (0x4BEFD2): pixel `x`, `dword_833DB6` into `[row + 4 x]`, two
/// halves (bytes +2, +3, then +0, +1): the turn of the half's line (identity, then
/// `set_line_turn`, `scale_rows3`); the other line's two ends and the pixel turned into
/// its frame (`turn_pair`, 80-bit differences); the slope between the ends; the pixel's y
/// worked out from its x by a shake of the seed (one draw of 0x3FFF, a second of 0xFFFF
/// when the x is not below the span: `x87::random_step`); turned back
/// (`set_line_turn_back`), rounded with the line's start added, the low bytes kept.
pub unsafe fn map_pixel(row: u32, x: u32) {
    unsafe {
        let lines = game_addr(LINES_RVA) as u32;
        let f = |o: u32| get(lines + o);
        game_write(PIXEL_RVA, x);
        let pixel: [u32; 2] = game_read(PIXEL_RVA);
        let turn = game_addr(TURN_RVA[0]) as u32;
        let shake: u32 = game_read(SHAKE_RVA);
        for half in HALVES {
            game_write(WORK_RVA, pixel);
            crate::scene::matrix3::identity_matrix3(turn);
            set_line_turn(lines + half.line, false);
            scale_rows3(turn);
            let (lx, ly) = (f(half.line), f(half.line + 4));
            let turned = |x: [u8; 10], y: [u8; 10]| turn_pair([y, x]);
            let a = turned(
                x87::sub_80(f(half.other), lx),
                x87::sub_80(f(half.other + 4), ly),
            );
            let b = turned(
                x87::sub_80(f(half.other + 0xC), lx),
                x87::sub_80(f(half.other + 0x10), ly),
            );
            let p = turned(
                x87::int_sub_80(pixel[0] as i32, lx),
                x87::int_sub_80(pixel[1] as i32, ly),
            );
            // 0x82E40A, _40E, then _416, _41A (0x82E412 is left).
            let ends = [rounded(a[1]), rounded(a[0]), rounded(b[1]), rounded(b[0])];
            game_write(ENDS_RVA, [ends[0], ends[1]]);
            game_write(ENDS_RVA + 0xC, [ends[2], ends[3]]);
            let (px, py) = (rounded(p[1]), rounded(p[0]));
            game_write(WORK_RVA, [px, py]);
            let mut span = x87::sub(ends[2], ends[0]);
            let rise = x87::div(x87::sub(ends[3], ends[1]), span);
            game_write(SLOPE_RVA, [span, rise]);
            let negative = span & 0x8000_0000 != 0;
            if negative {
                span ^= 0x8000_0000;
                game_write(SLOPE_RVA, span);
            }
            let status = x87::sub_compare(px, ends[0], negative, span);
            game_write(STATUS_RVA, status as u32);
            let mut seed = next_seed(game_read(SEED_RVA));
            let (mut r, mut c) = (seed & 0x3FFF, game_read::<u32>(half.constants[0]));
            if status != 0x100 {
                seed = next_seed(seed);
                (r, c) = (seed & 0xFFFF, game_read(half.constants[1]));
            }
            game_write(SEED_RVA, seed);
            let k: u32 = game_read(half.constants[2]);
            let y = x87::random_step(r as i32, [shake, c, px, ends[0], rise, ends[1], k]);
            game_write(WORK_RVA + 4, y);
            set_line_turn(lines + half.line, true);
            scale_rows3(turn);
            let q = turn_pair([extended(y), extended(px)]);
            let (qx, qy) = (rounded(q[1]), rounded(q[0]));
            let out = [x87::add_to_int(qx, lx), x87::add_to_int(qy, ly)];
            game_write(WORK_RVA, out);
            let at = row.wrapping_add(x.wrapping_mul(4)).wrapping_add(half.bytes);
            *(at as *mut u8) = out[0] as u8;
            *((at + 1) as *mut u8) = out[1] as u8;
        }
    }
}

/// The rows a shape entry may fill: x and length inside 0x100, rows inside 0x80.
unsafe fn rows_fit(entries: u32, index: u32) -> Result<(), Refused> {
    unsafe {
        let e = entries.wrapping_add(index.wrapping_mul(16));
        let (spans, row, count) = (get(e), get(e + 4) as i32, get(e + 8) as i32);
        if count > 0 && (row < 0 || row + count > 0x80) {
            return Err(BadInput("pixel rows outside the table").into());
        }
        for i in 0..count.max(0) as u32 {
            let (x, len) = (
                get(spans.wrapping_add(8 * i)) as i32,
                get(spans.wrapping_add(8 * i + 4)) as i32,
            );
            if len > 0 && (x < 0 || x + len > 0x100) {
                return Err(BadInput("pixel spans outside the table").into());
            }
        }
    }
    Ok(())
}

/// `marks::map_shape_rows` (0x4BE855): entry `index` of the list at `entries`: for each
/// of its rows (`dword_833DB6` the row, from the table's row in `dword_8445D2`)
/// `flt_8445E2` the row's squared distance (`x87::int_plus_less_squared`) and `map_pixel`
/// for each of its span's pixels. Refuses rows or spans outside the 0x100 x 0x80 table,
/// before writing.
pub unsafe fn map_shape_rows(entries: u32, index: u32) -> Result<(), Refused> {
    unsafe {
        rows_fit(entries, index)?;
        let e = entries.wrapping_add(index.wrapping_mul(16));
        let (spans, row, count) = (get(e), get(e + 4), get(e + 8));
        game_write(PIXEL_RVA + 4, row);
        let table: u32 = game_read(TABLE_RVA);
        let mut at = row.wrapping_mul(0x400).wrapping_add(table);
        for i in 0..(count as i32).max(0) as u32 {
            let (x, len) = (
                get(spans.wrapping_add(8 * i)),
                get(spans.wrapping_add(8 * i + 4)),
            );
            let y: u32 = game_read(PIXEL_RVA + 4);
            game_write(
                ROW_RVA,
                x87::int_plus_less_squared(
                    y as i32,
                    game_read(ROW_ADD_RVA),
                    game_read(ROW_LESS_RVA),
                ),
            );
            for n in 0..(len as i32).max(0) as u32 {
                map_pixel(at, x.wrapping_add(n));
            }
            game_write(PIXEL_RVA + 4, y.wrapping_add(1));
            at = at.wrapping_add(0x400);
        }
    }
    Ok(())
}

/// `marks::map_shape` (0x4BE83E): shape `shape` of the shapes at `shapes`:
/// `g_marksMaskValue` the shape, its two entries (2 n, 2 n + 1) of the rows at +8 mapped
/// (`map_shape_rows`). Refuses as `map_shape_rows`, before writing.
pub unsafe fn map_shape(shapes: u32, shape: u32) -> Result<(), Refused> {
    unsafe {
        let entries = get(shapes.wrapping_add(8));
        let first = shape.wrapping_add(shape);
        rows_fit(entries, first)?;
        rows_fit(entries, first.wrapping_add(1))?;
        game_write(crate::marks::masks::VALUE_RVA, shape);
        map_shape_rows(entries, first)?;
        map_shape_rows(entries, first.wrapping_add(1))?;
    }
    Ok(())
}

const MAPPED_RVA: [Slot; 3] = [Slot::at(0x844E10), Slot::at(0x844E18), Slot::at(0x844E1C)]; // shapes, table, corners
const CORNER_TABLE_RVA: Slot = Slot::at(0x8445E6); // dword_8445E6
const MIDDLE_RVA: Slot = Slot::at(0x8445D6); // flt_8445D6, flt_8445DA
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

/// `marks::map_shapes` (0x4BE752): with the x87 rounding toward zero (its word put back
/// after), the table at `table` (0x8000 dwords) cleared and made `g_marksPixelTable`, the
/// corner table `corners` kept, the shapes' middle (+0xC, +0x10) into 0x8445D6; for each
/// of the +4 shapes at `shapes` the line pair from corner entries k and k + 1 into
/// `g_marksLines` and `map_shape`. Refuses a count of 0 or less or past 0x80 (its loop
/// tests after counting), and rows or spans outside the table, before writing.
pub unsafe fn map_shapes(shapes: u32, table: u32, corners: u32) -> Result<(), Refused> {
    unsafe {
        let count = get(shapes + 4) as i32;
        if count <= 0 || count > 0x80 {
            return Err(BadInput("no shapes to map, or more than 0x80").into());
        }
        let entries = get(shapes + 8);
        for k in 0..2 * count as u32 {
            rows_fit(entries, k)?;
        }
        let came = x87::control_word();
        x87::set_control_word((came & 0xF3FF) | 0xC00);
        game_write(MAPPED_RVA[0], shapes);
        game_write(MAPPED_RVA[1], table);
        game_write(MAPPED_RVA[2], corners);
        std::ptr::write_bytes(table as *mut u8, 0, 0x2_0000);
        game_write(TABLE_RVA, game_read::<u32>(MAPPED_RVA[1]));
        let corners: u32 = game_read(MAPPED_RVA[2]);
        game_write(CORNER_TABLE_RVA, corners);
        game_write(
            MIDDLE_RVA,
            [
                x87::loaded(get(shapes + 0xC)),
                x87::loaded(get(shapes + 0x10)),
            ],
        );
        let lines = game_addr(LINES_RVA) as u32;
        for k in 0..count as u32 {
            let e = corners.wrapping_add(k * 0x18);
            for (to, from) in LINE_FROM {
                ((lines + to) as *mut u32).write_unaligned(get(e + from));
            }
            map_shape(shapes, k)?;
        }
        x87::set_control_word(came);
    }
    Ok(())
}
