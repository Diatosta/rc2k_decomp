//! `install::lay_out_pieces` (0x414C7E): a page's pieces (`dword_51C407` of them, 0x30 bytes
//! each after it: +0 the item's kind and marks, +1 the piece's kind, +2 its text, +4 its
//! item, +8 x, +0xC y, +0x10 width, +0x14 height) made from its list. The title (unless the
//! list's flags are 0x40 of 0xC0: `test; jg`) is kind 0x10 of `unk_51DD78`; then each item
//! not hidden (0x40 of 0xC0): a link a piece of kind 0; a choice a label (1) and a piece
//! (2) for each of its choices not hidden, its marks from the choice's high bits; a value
//! a label (1) and a piece (0x12); an edit the same, without the label when the item has
//! 0x10; a text kind 0x10 with its font's bits (`byte_51D52C`); an action 0x14. The places
//! and sizes are zeroed.
//!
//! In:  edi the list
//! Out: every register kept (`pusha` / `popa`); no flags read
//! Any other kind is fatal error 0x80A97451. More pieces than the 0xFFC bytes hold (85) is
//! refused (the original writes over the list variables after them). Caller
//! `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{fatal, rb, rw, wb, wd, ww};
use crate::install::vars::INSTALLER_PAGES;
use rc2k_core::install::layout;
use rc2k_core::install::pieces::{self, LayoutError, ListMemory};

pub const PIECE: u32 = layout::PIECE;

/// The game's lists, read where the original reads them.
struct GameList;

impl ListMemory for GameList {
    fn byte(&self, at: u32) -> u8 {
        rb(at)
    }
    fn word(&self, at: u32) -> u16 {
        rw(at)
    }
    fn dword(&self, at: u32) -> u32 {
        crate::install::installer::rd(at)
    }
    fn text_font(&self, font: u8) -> u8 {
        INSTALLER_PAGES.kind_fonts.get(4 + 8 * font as usize)
    }
}

pub fn lay_out_pieces(list: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::install::pieces::lay_out`; the pieces written here.
    let made = match pieces::lay_out(list, INSTALLER_PAGES.title.addr(), &GameList) {
        Ok(made) => made,
        Err(LayoutError::UnknownKind) => return Err(fatal(pieces::UNKNOWN_KIND)),
        Err(LayoutError::TooMany) => {
            return Err(BadInput("more pieces than the page's table holds").into());
        }
    };
    let base = crate::place::Place::from(&INSTALLER_PAGES.layout);
    for (n, piece) in made.iter().enumerate() {
        let at = base.add(4 + PIECE * n as u32);
        if let Some(text) = piece.text {
            ww(at.add(2), text);
        }
        wb(at, piece.marks);
        wb(at.add(1), piece.kind);
        wd(at.add(4), piece.item);
        for k in 0..4 {
            wd(at.add(8 + 4 * k), 0);
        }
    }
    wd(base, made.len() as u32);
    Ok(())
}
