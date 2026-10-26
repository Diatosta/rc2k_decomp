//! `install::place_row` (0x414F11): the pieces of one box of a page (`install::place_pieces`:
//! the pieces of one item row record `{x, y, width, height}`) placed in it: labels (bit 0 of
//! the piece's kind) in a column as wide as the widest, choices (bit 1) after their label
//! and each other 10 pixels apart, wrapped to a new line where they would pass the box's
//! width, the rest centred (or, with bit 2, at 0, with bit 3, at the right edge); then the
//! whole centred across and down in the box (by an even number of pixels down in the
//! title's box `unk_51DD98`), moved to its place.
//!
//! In:  ebx the first piece, ecx the pieces, esi the box, eax as it comes (a value the
//!      wrap reads before setting it)
//! Out: eax the box's x, ebx past the pieces, ecx 0, edx the centring down, edi the height,
//!      ebp the centring across; esi kept; no flags read
//! Caller `install::place_pieces`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{rb, rd, wd};
use crate::install::lay_out_pieces::PIECE;
use crate::install::vars::INSTALLER_PAGES;
use rc2k_core::install::layout::{self, Area, Piece};

/// `first` the first piece, `count` the pieces, `the_box` the box, `eax` as it comes (the
/// wrap reads it before setting it).
pub fn place_row(first: u32, count: u32, the_box: u32, eax: u32) -> Result<(), Refused> {
    let at = |k: u32| first.wrapping_add(PIECE * k);
    // Converted: `rc2k_core::install::layout::place_row`, on the pieces' fields it reads;
    // their x and y written back.
    let mut pieces: Vec<Piece> = (0..count)
        .map(|k| Piece {
            marks: rb(at(k)),
            kind: rb(at(k) + 1),
            x: rd(at(k) + 8),
            y: rd(at(k) + 0xC),
            width: rd(at(k) + 0x10),
            height: rd(at(k) + 0x14),
        })
        .collect();
    let area = Area {
        x: rd(the_box),
        y: rd(the_box + 4),
        width: rd(the_box + 8),
        height: rd(the_box + 0xC),
    };
    let title = the_box == INSTALLER_PAGES.title_box.addr();
    layout::place_row(&mut pieces, area, title, eax)?;
    for (k, p) in (0..).zip(&pieces) {
        wd(at(k) + 8, p.x);
        wd(at(k) + 0xC, p.y);
    }
    Ok(())
}
