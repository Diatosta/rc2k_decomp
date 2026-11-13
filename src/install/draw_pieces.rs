//! Each of a page's pieces drawn at its place: a plain text piece as its string in the font of
//! its marks; else by the item's kind: an edit its text (with the caret after it while it is
//! edited), a text its string and the item's appended text, an action its drawing action.
//! Refused: no pieces. Called by `install::run_page` (the surface locked).

use crate::errors::{BadInput, Refused};
use crate::install::installer::{
    draw_action, draw_string, draw_text, push_string_by_id, store_text, text_action, text_at,
};
use crate::install::page_records::{Piece, piece_count};
use crate::install::vars::InstallerPages;
use crate::strings::owner::Strings;
use rc2k_core::install::pieces::{self, Way};

fn font(
    install: &crate::install::owner::Install,
    index: u32,
) -> Result<crate::gfx::typefaces::FontId, BadInput> {
    crate::install::vars::InstallerPages::font_of(install, index)
}

fn colour(install: &crate::install::owner::Install, marks: u8, kind: u32) -> u32 {
    InstallerPages::kind_font(install, pieces::colour_at(marks, kind)) as u32
}

pub fn draw_pieces(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    install: &'static crate::install::owner::Install,
    strings: &'static Strings,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    let count = piece_count(install);
    if count == 0 {
        return Err(BadInput("a page of no pieces").into());
    }
    for k in 0..count {
        let piece = Piece::nth(install, k);
        let first = piece.marks();
        let (x, y) = (piece.x(), piece.y());
        match Way::of(first, piece.kind()) {
            Way::String { font_kind } => {
                draw_string(
                    so,
                    piece.text_id() as u32,
                    font(install, colour(install, first, font_kind))?,
                    x,
                    y,
                    gfx,
                )?;
            }
            Way::Edit => {
                let item = piece.item();
                let record = item.edit_field();
                let edit_font = font(install, colour(install, first, record.font_kind() as u32))?;
                let mut text = text_at(item.text());
                // The drawer reads to the NUL.
                if !pieces::caret(first, record.flags()) {
                    text.push(0);
                } else {
                    // The text and the caret, kept in the text buffer (anything reads it
                    // afterwards only after another page text overwrote it).
                    push_string_by_id(so, &mut text, 4);
                    text.push(0);
                    store_text(&strings.text.buffer.text, &text);
                }
                draw_text(edit_font, &text, x, y, gfx)?;
            }
            Way::Text => {
                let item = piece.item();
                let mut text = Vec::new();
                push_string_by_id(so, &mut text, piece.text_id() as u32);
                let action = item.data();
                if action != 0 {
                    text_action(action, &mut text)?;
                }
                text.push(0);
                store_text(&strings.text.buffer.text, &text);
                draw_text(
                    font(install, colour(install, first, item.font_kind() as u32))?,
                    &text,
                    x,
                    y,
                    gfx,
                )?;
            }
            Way::Action => draw_action(piece.item().data())?,
            Way::Other => {}
        }
    }
    Ok(())
}
