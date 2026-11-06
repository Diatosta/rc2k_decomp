//! `make_masks`' rasteriser (IDA, 2026-10-17): a quad split into two triangles, each
//! triangle's corners sorted by height and its two halves turned into rows of spans (x and
//! length, whole numbers) at the masks' cursor in the work block, with a run record (spans,
//! first row, count) for `draw_mask`. The triangle sets the x87 to round toward zero for
//! all of it.

use crate::effects::mask_store::Masks;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::session::{Float, read_bits, write_bits};

/// The edge pair `span_edges` works from (`MASKS.edge`): y0, y1, the left and right x at y0, at y1.
/// The control words (`MASKS.control_word`, `MASKS.status_word`): the one the triangle came
/// with, then its own (later the compares' status words).
const WORK_LEN: u32 = 0x80000;
const MOST_ROWS: u32 = 0x1000;

/// The dword at byte `at` of the runs' table (`MASKS.run_table`; the runs, and the shapes'
/// `rows`, are offsets in it).
pub fn run_get(masks: &Masks, at: u32) -> Result<u32, BadInput> {
    let bytes = masks
        .vars
        .run_table
        .read_at(at as usize, 4, "the mask runs")?;
    Ok(u32::from_le_bytes(bytes.try_into().expect("four bytes")))
}

/// `v` put at byte `at` of the runs' table.
fn run_put(masks: &Masks, at: u32, v: u32) -> Result<(), BadInput> {
    masks
        .vars
        .run_table
        .write_at(at as usize, &v.to_le_bytes(), "the mask runs")
}

/// The rows the edge pair's loop runs (its check: the row rounded below y1 rounded, the
/// row counting up by `flt_89C9AC`), or `None` past `MOST_ROWS`.
pub fn edge_rows(edge: &[u32; 6]) -> Option<u32> {
    let one: u32 = crate::effects::owner::effects().setup.mask_constants.ones[2].bits();
    let mut row = x87::round_float(edge[0]);
    let end = x87::to_int(edge[1]);
    let mut rows = 0;
    loop {
        let r = x87::to_int(row);
        row = x87::add(row, one);
        if r >= end {
            return Some(rows);
        }
        rows += 1;
        if rows > MOST_ROWS {
            return None;
        }
    }
}

/// Refuses rows that would run past the work block (or away), from `cursor` (a byte
/// offset in `effects::masks::MASK_WORK`).
fn room(cursor: u32, rows: Option<u32>) -> Result<(), Refused> {
    match rows {
        Some(n) if cursor as u64 + 8 * n as u64 <= WORK_LEN as u64 => Ok(()),
        Some(_) => Err(BadInput("mask spans past the work block").into()),
        None => Err(BadInput("mask spans that run away (more than 0x1000 rows)").into()),
    }
}

/// `effects::span_edges` (0x4BE315): from the edge pair (`dword_82FD16` ..), a row of spans
/// at `to` (a byte offset in the work; 8 bytes: x, length) for each whole row from y0 rounded up to y1: both x's at
/// the row's middle by the slopes, the two put in order (swapped also when equal), the left
/// rounded and the length the right less it. The x87's rounding as it is set (the triangle
/// sets toward zero). Gives where the next row would go (the original's edi). Refuses
/// rows past the work block or more than 0x1000, before writing.
pub fn span_edges(masks: &Masks, to: u32) -> Result<u32, Refused> {
    let mut to = to;
    {
        let e: [u32; 6] = read_bits(&masks.vars.edge);
        room(to, edge_rows(&e))?;
        let [y0, y1, xa0, xb0, xa1, xb1] = e;
        let ones: [u32; 3] = read_bits(&crate::effects::owner::effects().setup.mask_constants.ones);
        let height = x87::sub(y1, y0);
        masks.vars.height.set_bits(height);
        let slopes = [
            x87::sub_div(xa1, xa0, height),
            x87::sub_div(xb1, xb0, height),
        ];
        write_bits(&masks.vars.slopes, slopes);
        let (n, mut xa) = x87::frac_step(y0, ones[0], slopes[0], xa0);
        masks.vars.row_int.set(n as u32);
        let (n, mut xb) = x87::frac_step(y0, ones[1], slopes[1], xb0);
        masks.vars.row_int.set(n as u32);
        let mut row = x87::round_float(y0);
        loop {
            let r = x87::to_int(row);
            let end = x87::to_int(y1);
            crate::effects::owner::effects()
                .setup
                .work
                .scratch
                .set(end as u32);
            row = x87::add(row, ones[2]);
            if r >= end {
                break;
            }
            let mut a = x87::loaded(xa);
            xa = x87::add(xa, slopes[0]);
            let mut b = x87::loaded(xb);
            xb = x87::add(xb, slopes[1]);
            let status = x87::compare_status(a, b);
            masks.vars.status_word.set(status);
            if status == 0 {
                (a, b) = (b, a);
            }
            let left = x87::to_int(a);
            crate::effects::masks::work_put(masks, to, left as u32)?;
            crate::effects::masks::work_put(
                masks,
                to + 4,
                x87::float_less_int_to_int(b, left) as u32,
            )?;
            to += 8;
        }
        masks.vars.row.set_bits(row);
        write_bits(&masks.vars.xs, [xa, xb]);
    }
    Ok(to)
}

/// The corners sorted by y, the original's way: pass after pass, a pair swapped when the
/// second's y compares below the first's. Also the last compare's status (C0 .. C2), which
/// the original leaves in `word_82FCE8`.
fn sorted(mut t: [u32; 6]) -> ([u32; 6], u16) {
    loop {
        let mut swapped = false;
        let mut status = 0;
        for c in 0..2 {
            status = x87::compare_status(t[2 * c + 3], t[2 * c + 1]);
            if status == 0x100 {
                swapped = true;
                t.swap(2 * c, 2 * c + 2);
                t.swap(2 * c + 1, 2 * c + 3);
            }
        }
        if !swapped {
            return (t, status);
        }
    }
}

/// A triangle's two halves' edge pairs after sorting, and the split x (the x87 set as the
/// triangle sets it).
fn halves(t: &[u32; 6]) -> ([[u32; 6]; 2], u32) {
    let [x0, y0, x1, y1, x2, y2] = *t;
    let split = x87::split_x(y0, y1, y2, x0, x2);
    (
        [[y0, y1, x0, x0, x1, split], [y1, y2, x1, split, x2, x2]],
        split,
    )
}

/// The rows a triangle's spans take (its corners as given, sorted here), or `None` when
/// one half would run away; with the x87 rounding toward zero, as `span_triangle` has it.
pub fn triangle_rows(corners: [u32; 6]) -> Option<u32> {
    let came = x87::control_word();
    x87::set_control_word((came & 0xF3FF) | 0xC00);
    let (h, _) = halves(&sorted(corners).0);
    let rows = match (edge_rows(&h[0]), edge_rows(&h[1])) {
        (Some(a), Some(b)) => Some(a + b),
        _ => None,
    };
    x87::set_control_word(came);
    rows
}

/// Refuses `triangles` runs or `rows` spans (`None`: away) past their room.
pub fn spans_fit(masks: &Masks, triangles: u32, rows: Option<u32>) -> Result<(), Refused> {
    let runs: u32 = masks.vars.runs.get();
    if runs as u64 + 16 * triangles as u64 > 0x2000 {
        return Err(BadInput("mask runs past their table").into());
    }
    room(masks.vars.cursor.get(), rows)
}

/// `effects::span_triangle` (0x4BE14D): the triangle `at` (three x, y floats) copied to
/// 0x833D2E and sorted by y; a run (`g_marksMaskRuns`, 16 bytes on) at the cursor; its top
/// half and its bottom (split at the middle corner's height, x by `x87::split_x`) spanned
/// (`span_edges`); the run's first row (the top y rounded) and count noted, the cursor on,
/// `dword_82E342` counting the spans. All with the x87 rounding toward zero; its control
/// word put back after. Refuses runs past their table, and spans past the work block or
/// away, before writing.
pub fn span_triangle(masks: &Masks, at: &[Float; 6]) -> Result<(), Refused> {
    {
        let came = x87::control_word();
        let own = (came & 0xF3FF) | 0xC00;
        x87::set_control_word(own);
        let (t, status) = sorted(read_bits(at));
        let y0 = t[1];
        let (halves, split) = halves(&t);
        let runs: u32 = masks.vars.runs.get();
        let cursor: u32 = masks.vars.cursor.get();
        let rows = match (edge_rows(&halves[0]), edge_rows(&halves[1])) {
            (Some(a), Some(b)) => Some(a + b),
            _ => None,
        };
        let fits = spans_fit(masks, 1, rows);
        if let Err(e) = fits {
            x87::set_control_word(came);
            return Err(e);
        }
        // The sort's compares write their status over the own word (`fstsw`).
        masks.vars.control_word.set(came);
        masks.vars.status_word.set(status);
        write_bits(&masks.vars.triangle, t);
        masks.vars.runs.set(runs + 16);
        run_put(masks, runs, cursor)?;
        let mut to = cursor;
        masks.vars.split.set_bits(split);
        for half in halves {
            write_bits(&masks.vars.edge, half);
            to = span_edges(masks, to)?;
        }
        masks.vars.cursor.set(to);
        let count = (to - cursor) >> 3;
        // Through `dword_71C880`, as the original's `fistp`.
        let first = x87::to_int(y0) as u32;
        crate::effects::owner::effects()
            .setup
            .work
            .scratch
            .set(first);
        run_put(masks, runs + 4, first)?;
        run_put(masks, runs + 8, count)?;
        masks
            .vars
            .spans_total
            .set(masks.vars.spans_total.get().wrapping_add(count));
        x87::set_control_word(came);
    }
    Ok(())
}

/// `effects::span_quad` (0x4BE0C6): the quad `at` (four x, y floats) as two triangles, its
/// corners 0, 1, 2 and then 2, 3, 0 (through 0x833D46), each `span_triangle`d. Refuses as
/// `span_triangle` (the second only after the first is made).
pub fn span_quad(masks: &Masks, at: &[Float; 8]) -> Result<(), Refused> {
    {
        let q: [u32; 8] = read_bits(at);
        for corners in [[0usize, 1, 2], [2, 3, 0]] {
            let mut t = [0u32; 6];
            for (k, &c) in corners.iter().enumerate() {
                t[2 * k] = q[2 * c];
                t[2 * k + 1] = q[2 * c + 1];
            }
            write_bits(&masks.vars.quad_triangle, t);
            span_triangle(masks, &masks.vars.quad_triangle)?;
        }
    }
    Ok(())
}
