//! `make_masks`' rasteriser (IDA, 2026-10-17): a quad split into two triangles, each
//! triangle's corners sorted by height and its two halves turned into rows of spans (x and
//! length, whole numbers) at the masks' cursor in the work block, with a run record (spans,
//! first row, count) for `draw_mask`. The triangle sets the x87 to round toward zero for
//! all of it.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::marks::masks::{CURSOR_RVA, RUNS_RVA, WORK_RVA};
use crate::math::x87;
use crate::service::harness::{Region, at};

/// The edge pair `span_edges` works from: y0, y1, the left and right x at y0, at y1.
pub const EDGE_RVA: usize = rva(0x82FD16); // dword_82FD16 .. dword_82FD2A
const HEIGHT_RVA: usize = rva(0x82FCEE); // flt_82FCEE
const ROW_INT_RVA: usize = rva(0x82FCF2); // dword_82FCF2
const ROW_RVA: usize = rva(0x82FCF6); // flt_82FCF6
const SLOPES_RVA: usize = rva(0x82FCFA); // flt_82FCFA, flt_82FCFE
const XS_RVA: usize = rva(0x82FD02); // flt_82FD02, flt_82FD06
const SPLIT_RVA: usize = rva(0x82FD0A); // flt_82FD0A
const ONES_RVA: usize = rva(0x89C9A4); // flt_89C9A4 .. flt_89C9AC
const SCRATCH_RVA: usize = rva(0x71C880); // dword_71C880
/// The control words: the one the triangle came with, then its own (later the compares'
/// status words).
pub const WORDS_RVA: usize = rva(0x82FCE6); // word_82FCE6, word_82FCE8
pub const TRIANGLE_RVA: usize = rva(0x833D2E); // dword_833D2E: the sorted corners
pub const QUAD_TRIANGLE_RVA: usize = rva(0x833D46); // dword_833D46: a quad's triangle
pub const SPANS_TOTAL_RVA: usize = rva(0x82E342); // dword_82E342
const RUNS_START: u32 = 0x831D2E;
const RUNS_END: u32 = 0x833D2E;
const WORK_LEN: u32 = 0x80000;
const MOST_ROWS: u32 = 0x1000;

unsafe fn put(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) }
}

/// The rows the edge pair's loop runs (its check: the row rounded below y1 rounded, the
/// row counting up by `flt_89C9AC`), or `None` past `MOST_ROWS`.
pub fn edge_rows(edge: &[u32; 6]) -> Option<u32> {
    let one: u32 = unsafe { game_read(ONES_RVA + 8) };
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

/// Refuses rows that would run past the work block (or away), from `cursor`.
unsafe fn room(cursor: u32, rows: Option<u32>) -> Result<(), Refused> {
    let work: u32 = unsafe { game_read(WORK_RVA) };
    match rows {
        Some(n)
            if cursor >= work && cursor as u64 + 8 * n as u64 <= work as u64 + WORK_LEN as u64 =>
        {
            Ok(())
        }
        Some(_) => Err(BadInput("mask spans past the work block").into()),
        None => Err(BadInput("mask spans that run away (more than 0x1000 rows)").into()),
    }
}

/// `marks::span_edges` (0x4BE315): from the edge pair (`dword_82FD16` ..), a row of spans
/// at edi (8 bytes: x, length) for each whole row from y0 rounded up to y1: both x's at
/// the row's middle by the slopes, the two put in order (swapped also when equal), the left
/// rounded and the length the right less it. The x87's rounding as it is set (the triangle
/// sets toward zero). Leaves eax and ebx the last check's row and end, edi past the rows.
/// Refuses rows past the work block or more than 0x1000, before writing.
pub unsafe fn span_edges(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let e: [u32; 6] = game_read(EDGE_RVA);
        room(g.edi, edge_rows(&e))?;
        let [y0, y1, xa0, xb0, xa1, xb1] = e;
        let ones: [u32; 3] = game_read(ONES_RVA);
        let height = x87::sub(y1, y0);
        game_write(HEIGHT_RVA, height);
        let slopes = [
            x87::sub_div(xa1, xa0, height),
            x87::sub_div(xb1, xb0, height),
        ];
        game_write(SLOPES_RVA, slopes);
        let (n, mut xa) = x87::frac_step(y0, ones[0], slopes[0], xa0);
        game_write(ROW_INT_RVA, n);
        let (n, mut xb) = x87::frac_step(y0, ones[1], slopes[1], xb0);
        game_write(ROW_INT_RVA, n);
        let mut row = x87::round_float(y0);
        loop {
            g.eax = x87::to_int(row) as u32;
            g.ebx = x87::to_int(y1) as u32;
            game_write(SCRATCH_RVA, g.ebx);
            row = x87::add(row, ones[2]);
            if g.eax as i32 >= g.ebx as i32 {
                break;
            }
            let mut a = x87::loaded(xa);
            xa = x87::add(xa, slopes[0]);
            let mut b = x87::loaded(xb);
            xb = x87::add(xb, slopes[1]);
            let status = x87::compare_status(a, b);
            game_write(WORDS_RVA + 2, status);
            if status == 0 {
                (a, b) = (b, a);
            }
            let left = x87::to_int(a);
            put(g.edi, left as u32);
            put(g.edi + 4, x87::float_less_int_to_int(b, left) as u32);
            g.edi += 8;
        }
        game_write(ROW_RVA, row);
        game_write(XS_RVA, [xa, xb]);
    }
    Ok(g)
}

/// What `span_edges` writes besides the spans.
pub fn edge_regions() -> Vec<Region> {
    let a = |r: usize| game_addr(r) as u32;
    vec![(a(HEIGHT_RVA), 0x20), (a(WORDS_RVA), 4), at(SCRATCH_RVA)]
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
pub unsafe fn spans_fit(triangles: u32, rows: Option<u32>) -> Result<(), Refused> {
    unsafe {
        let runs: u32 = game_read(RUNS_RVA);
        let start = game_addr(rva(RUNS_START as usize)) as u32;
        let end = game_addr(rva(RUNS_END as usize)) as u32;
        if runs < start || runs as u64 + 16 * triangles as u64 > end as u64 {
            return Err(BadInput("mask runs past their table").into());
        }
        room(game_read(CURSOR_RVA), rows)
    }
}

/// `marks::span_triangle` (0x4BE14D): the triangle at ebp (three x, y floats) copied to
/// 0x833D2E and sorted by y; a run (`g_marksMaskRuns`, 16 bytes on) at the cursor; its top
/// half and its bottom (split at the middle corner's height, x by `x87::split_x`) spanned
/// (`span_edges`); the run's first row (the top y rounded) and count noted, the cursor on,
/// `dword_82E342` counting the spans. All with the x87 rounding toward zero; its control
/// word put back after. Every register kept. Refuses runs past their table, and spans past
/// the work block or away, before writing.
pub unsafe fn span_triangle(entry: &Regs) -> Result<Regs, Refused> {
    unsafe {
        let came = x87::control_word();
        let own = (came & 0xF3FF) | 0xC00;
        x87::set_control_word(own);
        let (t, status) = sorted((entry.ebp as *const [u32; 6]).read_unaligned());
        let y0 = t[1];
        let (halves, split) = halves(&t);
        let runs: u32 = game_read(RUNS_RVA);
        let cursor: u32 = game_read(CURSOR_RVA);
        let rows = match (edge_rows(&halves[0]), edge_rows(&halves[1])) {
            (Some(a), Some(b)) => Some(a + b),
            _ => None,
        };
        let fits = spans_fit(1, rows);
        if let Err(e) = fits {
            x87::set_control_word(came);
            return Err(e);
        }
        // The sort's compares write their status over the own word (`fstsw`).
        game_write(WORDS_RVA, [came, status]);
        game_write(TRIANGLE_RVA, t);
        game_write(RUNS_RVA, runs + 16);
        put(runs, cursor);
        let mut g = Regs {
            esi: runs,
            edi: cursor,
            ebp: game_addr(TRIANGLE_RVA) as u32,
            ..*entry
        };
        game_write(SPLIT_RVA, split);
        for half in halves {
            game_write(EDGE_RVA, half);
            g = span_edges(&g)?;
        }
        game_write(CURSOR_RVA, g.edi);
        let count = (g.edi - cursor) >> 3;
        // Through `dword_71C880`, as the original's `fistp`.
        let first = x87::to_int(y0) as u32;
        game_write(SCRATCH_RVA, first);
        put(runs + 4, first);
        put(runs + 8, count);
        game_write(
            SPANS_TOTAL_RVA,
            game_read::<u32>(SPANS_TOTAL_RVA).wrapping_add(count),
        );
        x87::set_control_word(came);
    }
    Ok(*entry)
}

/// What `span_triangle` writes besides the spans and the run.
pub fn triangle_regions() -> Vec<Region> {
    let a = |r: usize| game_addr(r) as u32;
    let mut r = edge_regions();
    r.extend([
        (a(EDGE_RVA), 24),
        at(SPLIT_RVA),
        (a(TRIANGLE_RVA), 24),
        at(RUNS_RVA),
        at(CURSOR_RVA),
        at(SPANS_TOTAL_RVA),
    ]);
    r
}

/// `marks::span_quad` (0x4BE0C6): the quad at ebp (four x, y floats) as two triangles, its
/// corners 0, 1, 2 and then 2, 3, 0 (through 0x833D46), each `span_triangle`d. Every
/// register kept. Refuses as `span_triangle` (the second only after the first is made).
pub unsafe fn span_quad(entry: &Regs) -> Result<Regs, Refused> {
    unsafe {
        let q = (entry.ebp as *const [u32; 8]).read_unaligned();
        for corners in [[0usize, 1, 2], [2, 3, 0]] {
            let mut t = [0u32; 6];
            for (k, &c) in corners.iter().enumerate() {
                t[2 * k] = q[2 * c];
                t[2 * k + 1] = q[2 * c + 1];
            }
            game_write(QUAD_TRIANGLE_RVA, t);
            span_triangle(&Regs {
                ebp: game_addr(QUAD_TRIANGLE_RVA) as u32,
                ..*entry
            })?;
        }
    }
    Ok(*entry)
}

// ---- for the self-tests ----

/// A work block of the test's in `g_marksMaskWork` with the cursor at its start and the
/// runs at their table's start; the game's variables put back when dropped.
pub struct TestWork {
    pub block: Vec<u8>,
    inputs: Vec<Region>,
    saved: Vec<u8>,
}

impl TestWork {
    pub fn new() -> Self {
        let a = |r: usize| game_addr(r) as u32;
        let mut inputs = triangle_regions();
        inputs.extend([
            at(WORK_RVA),
            at(crate::marks::masks::RUN_COUNT_RVA),
            (a(QUAD_TRIANGLE_RVA), 24),
            (a(rva(RUNS_START as usize)), 0x100),
        ]);
        let saved = crate::service::harness::capture(&inputs);
        Self {
            block: vec![0xAA; 0x4000],
            inputs,
            saved,
        }
    }

    pub fn addr(&self) -> u32 {
        self.block.as_ptr() as u32
    }

    /// The block 0xAA, the cursor at its start, the runs at theirs, the counts 0.
    pub fn reset(&self) {
        unsafe {
            std::ptr::write_bytes(self.addr() as *mut u8, 0xAA, self.block.len());
            game_write(WORK_RVA, self.addr());
            game_write(CURSOR_RVA, self.addr());
            game_write(RUNS_RVA, game_addr(rva(RUNS_START as usize)) as u32);
            game_write(SPANS_TOTAL_RVA, 0u32);
            game_write(crate::marks::masks::RUN_COUNT_RVA, 0u32);
        }
    }

    /// What the tests compare.
    pub fn regions(&self) -> Vec<Region> {
        let mut r = triangle_regions();
        r.push((self.addr(), self.block.len()));
        r.push((game_addr(rva(RUNS_START as usize)) as u32, 0x100));
        r.push((game_addr(QUAD_TRIANGLE_RVA) as u32, 24));
        r
    }
}

impl Default for TestWork {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestWork {
    fn drop(&mut self) {
        crate::service::harness::restore(&self.inputs, &self.saved);
    }
}

/// `n` corners (x, y floats) by lot inside a mask, now and then two at one height or on a
/// whole row.
pub fn test_corners(seed: u32, n: usize) -> Vec<u32> {
    let mut s = seed;
    let mut next = move || {
        s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
        s >> 8
    };
    let mut v = Vec::new();
    for _ in 0..n {
        let x = (next() % 0xF000) as f32 / 256.0;
        let y = match next() % 6 {
            0 => (next() % 0x70) as f32,
            _ => (next() % 0x7000) as f32 / 256.0,
        };
        v.extend([x.to_bits(), y.to_bits()]);
    }
    if seed.is_multiple_of(3) {
        v[3] = v[1];
    }
    v
}
