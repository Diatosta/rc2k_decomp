//! `install::draw_pieces` (0x415058): each of a page's pieces drawn at its place. A plain
//! text piece (no 0x10 in its kind) its string (`text::draw_string_by_id`) in the font of
//! its marks (`byte_51D528`: bits 6, 7 of its first byte, its kind and 0x10); else by the
//! item's kind: an edit its text (`text::draw_text`) in the font of its record's kind and
//! marks, with the caret (string 4) after it while it is edited, lit and its record has
//! bit 1; a text its string and the item's appended text in `text`; an action its drawing
//! action (`[item+0xC]`, ecx and edx the place).
//!
//! In:  nothing (the pieces at `dword_51C407`)
//! Out: every register kept (`pusha` / `popa`); no flags read
//! No pieces would go round 2^32 times: refused. Caller `install::run_page` (the surface
//! locked).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{
    append_byte, append_string_by_id, draw_action, draw_string, draw_text, str_append, text_action,
};
use crate::install::page_records::{Piece, piece_count};
use crate::install::vars::InstallerPages;
use crate::place::Place;
use crate::strings::owner::strings;
use rc2k_core::install::pieces::{self, Way};

fn font(index: u32) -> Result<crate::gfx::typefaces::FontId, BadInput> {
    crate::install::vars::InstallerPages::font_of(index)
}

fn colour(marks: u8, kind: u32) -> u32 {
    InstallerPages::kind_font(pieces::colour_at(marks, kind)) as u32
}

pub fn draw_pieces() -> Result<(), Refused> {
    let strings = strings();
    let count = piece_count();
    if count == 0 {
        return Err(BadInput("a page of no pieces").into());
    }
    for k in 0..count {
        let piece = Piece::nth(k);
        let first = piece.marks();
        let (x, y) = (piece.x(), piece.y());
        // Converted: `rc2k_core::install::pieces` (`Way`, `colour_at`, `caret`).
        match Way::of(first, piece.kind()) {
            Way::String { font_kind } => {
                draw_string(
                    piece.text_id() as u32,
                    font(colour(first, font_kind))?.game_ptr(),
                    x,
                    y,
                )?;
            }
            Way::Edit => {
                let item = piece.item();
                let record = item.edit_field();
                let edit_font = font(colour(first, record.font_kind() as u32))?;
                let mut text = item.text();
                if pieces::caret(first, record.flags()) {
                    let cursor = str_append(text, &strings.text.buffer.text);
                    let cursor = append_string_by_id(4, cursor);
                    append_byte(cursor, 0);
                    text = Place::from(&strings.text.buffer.text);
                }
                draw_text(edit_font, text, x, y)?;
            }
            Way::Text => {
                let item = piece.item();
                let mut cursor =
                    append_string_by_id(piece.text_id() as u32, &strings.text.buffer.text);
                let action = item.data();
                if action != 0 {
                    cursor = text_action(action, cursor)?;
                }
                append_byte(cursor, 0);
                draw_text(
                    font(colour(first, item.font_kind() as u32))?,
                    &strings.text.buffer.text,
                    x,
                    y,
                )?;
            }
            Way::Action => draw_action(piece.item().data())?,
            Way::Other => {}
        }
    }
    Ok(())
}
