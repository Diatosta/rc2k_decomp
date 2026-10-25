//! `install::place_pieces` (0x414ED1): a page's pieces placed, box by box: the pieces in a
//! row whose items share a box (the item's +4: `{x, y, width, height}`; none: the title's
//! box `unk_51DD98`) go to `install::place_row` together.
//!
//! In:  nothing (the pieces at `dword_51C407`)
//! Out: every register kept (`pusha` / `popa`); no flags read
//! No pieces would go round 2^32 times: refused. Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::rd;
use crate::install::lay_out_pieces::PIECE;
use crate::install::vars::INSTALLER_PAGES;

pub fn place_pieces() -> Result<(), Refused> {
    let base = INSTALLER_PAGES.layout.addr() as u32 + 4;
    let at = |k: u32| base.wrapping_add(PIECE * k);
    // Converted: `rc2k_core::install::layout::rows` (each piece's box read through its item).
    let rows = rc2k_core::install::layout::rows(INSTALLER_PAGES.layout.get_u32(0), |k| {
        rd(rd(at(k) + 4) + 4)
    })?;
    for (first, n, the_box) in rows {
        let place = if the_box == 0 {
            INSTALLER_PAGES.title_box.addr() as u32
        } else {
            the_box
        };
        crate::install::place_row::place_row(at(first), n, place, the_box)?;
    }
    Ok(())
}
