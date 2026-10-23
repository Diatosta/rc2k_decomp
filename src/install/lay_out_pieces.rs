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
//! `install::run_page`. Shadow mode compares. Self-test: lists of the test's against the
//! original (the fatal kind only on the Rust version).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::install::installer::{LAYOUT, a, fatal, rb, rw, wb, wd, ww};
use crate::race::viewer::regs_entry;
use rc2k_core::install::layout;
use rc2k_core::install::pieces::{self, LayoutError, ListMemory};

pub const PIECE: u32 = layout::PIECE;
const TITLE: usize = 0x51DD78; // unk_51DD78: the title's item
const TEXT_FONTS: usize = 0x51D52C; // byte_51D52C

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
        rb(a(TEXT_FONTS) + 8 * font as u32)
    }
}

pub fn lay_out_pieces(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::install::pieces::lay_out`; the pieces written here.
    let made = match pieces::lay_out(entry.edi, a(TITLE), &GameList) {
        Ok(made) => made,
        Err(LayoutError::UnknownKind) => return Err(fatal(pieces::UNKNOWN_KIND)),
        Err(LayoutError::TooMany) => {
            return Err(BadInput("more pieces than the page's table holds").into());
        }
    };
    let base = a(LAYOUT);
    for (n, piece) in made.iter().enumerate() {
        let at = base + 4 + PIECE * n as u32;
        if let Some(text) = piece.text {
            ww(at + 2, text);
        }
        wb(at, piece.marks);
        wb(at + 1, piece.kind);
        wd(at + 4, piece.item);
        for k in 0..4 {
            wd(at + 8 + 4 * k, 0);
        }
    }
    wd(base, made.len() as u32);
    Ok(*entry)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &lay_out_pieces)
}
