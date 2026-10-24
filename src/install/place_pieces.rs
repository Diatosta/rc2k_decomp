//! `install::place_pieces` (0x414ED1): a page's pieces placed, box by box: the pieces in a
//! row whose items share a box (the item's +4: `{x, y, width, height}`; none: the title's
//! box `unk_51DD98`) go to `install::place_row` together.
//!
//! In:  nothing (the pieces at `dword_51C407`)
//! Out: every register kept (`pusha` / `popa`); no flags read
//! No pieces would go round 2^32 times: refused. Caller `install::run_page`. Shadow mode
//! compares. Self-test: pages laid out by `install::lay_out_pieces` from lists of the
//! test's, measured as the test says, against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{LAYOUT, a, g, rd};
use crate::install::lay_out_pieces::PIECE;

const TITLE_BOX: usize = 0x51DD98; // unk_51DD98

pub fn place_pieces() -> Result<(), Refused> {
    let base = a(LAYOUT) + 4;
    let at = |k: u32| base.wrapping_add(PIECE * k);
    // Converted: `rc2k_core::install::layout::rows` (each piece's box read through its item).
    let rows = rc2k_core::install::layout::rows(g(LAYOUT), |k| rd(rd(at(k) + 4) + 4))?;
    for (first, n, the_box) in rows {
        let place = if the_box == 0 { a(TITLE_BOX) } else { the_box };
        crate::install::place_row::place_row(at(first), n, place, the_box)?;
    }
    Ok(())
}
