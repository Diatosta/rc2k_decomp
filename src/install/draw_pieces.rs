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
    append_byte, append_string_by_id, draw_action, draw_string, draw_text, rb, rd, rw, str_append,
    text_action,
};
use crate::install::lay_out_pieces::PIECE;
use crate::install::vars::INSTALLER_PAGES;
use crate::strings::text_buffer::TEXT_BUFFER;
use rc2k_core::install::pieces::{self, Way};

fn font(index: u32) -> u32 {
    rd(crate::session::address_of(&INSTALLER_PAGES.fonts) as u32 + 4 * index)
}

fn colour(marks: u8, kind: u32) -> u32 {
    rb(INSTALLER_PAGES.kind_fonts.addr() as u32 + pieces::colour_at(marks, kind)) as u32
}

pub fn draw_pieces() -> Result<(), Refused> {
    let count = INSTALLER_PAGES.layout.get_u32(0);
    if count == 0 {
        return Err(BadInput("a page of no pieces").into());
    }
    for k in 0..count {
        let piece = INSTALLER_PAGES.layout.addr() as u32 + 4 + PIECE * k;
        let first = rb(piece);
        let (x, y) = (rd(piece + 8), rd(piece + 0xC));
        // Converted: `rc2k_core::install::pieces` (`Way`, `colour_at`, `caret`).
        match Way::of(first, rb(piece + 1)) {
            Way::String { font_kind } => {
                draw_string(rw(piece + 2) as u32, font(colour(first, font_kind)), x, y)?;
            }
            Way::Edit => {
                let item = rd(piece + 4);
                let record = rd(item + 0xC);
                let edit_font = font(colour(first, rb(record + 4) as u32));
                let mut text = rd(item + 8);
                if pieces::caret(first, rb(record)) {
                    let cursor = str_append(text, TEXT_BUFFER.text.addr() as u32);
                    let cursor = append_string_by_id(4, cursor);
                    append_byte(cursor, 0);
                    text = TEXT_BUFFER.text.addr() as u32;
                }
                draw_text(edit_font, text, x, y)?;
            }
            Way::Text => {
                let item = rd(piece + 4);
                let mut cursor =
                    append_string_by_id(rw(piece + 2) as u32, TEXT_BUFFER.text.addr() as u32);
                let action = rd(item + 0xC);
                if action != 0 {
                    cursor = text_action(action, cursor)?;
                }
                append_byte(cursor, 0);
                draw_text(
                    font(colour(first, rb(item + 1) as u32)),
                    TEXT_BUFFER.text.addr() as u32,
                    x,
                    y,
                )?;
            }
            Way::Action => draw_action(rd(rd(piece + 4) + 0xC))?,
            Way::Other => {}
        }
    }
    Ok(())
}
