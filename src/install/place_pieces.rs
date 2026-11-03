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
use crate::install::page_records::{BoxRecord, Piece, piece_count};
use crate::install::vars::INSTALLER_PAGES;
use crate::place::Place;

pub fn place_pieces() -> Result<(), Refused> {
    // Converted: `rc2k_core::install::layout::rows` (each piece's box read through its item).
    let rows = rc2k_core::install::layout::rows(piece_count(), |k| Piece::nth(k).item().box_at())?;
    for (first, n, the_box) in rows {
        let title_box = Place::from(&INSTALLER_PAGES.title_box);
        let (place, title) = if the_box == 0 {
            (title_box, true)
        } else {
            (Place::from(the_box), the_box == title_box.game())
        };
        crate::install::place_row::place_row(first, n, BoxRecord::of(place), title, the_box)?;
    }
    Ok(())
}
