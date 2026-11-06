//! `install::measure_pieces` (0x414DF5): each of a page's pieces measured into its width
//! (+0x10) and height (+0x14). A piece whose item has 0x20 first has bits 0, 1 of its kind
//! moved to bits 2, 3. A piece of a plain text kind (no 0x10) is its string measured
//! (`text::measure_string_by_id`) in the font of its marks and kind (`byte_51D52D`,
//! `dword_51D4C4`); else by the item's kind: an edit its record's width and its font's
//! height (`byte_51D52F`); a text its string and the item's appended text (`[item+0xC]`
//! with eax `[item+8]`, if any) in `text`, measured (`text::measure_text`); an action its
//! box's size; any other what ecx and edx hold.
//!
//! In:  nothing (the pieces at `dword_51C407`)
//! Out: every register kept (`pusha` / `popa`); no flags read
//! No pieces would go round 2^32 times: refused. Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{
    append_byte, append_string_by_id, measure_string, measure_text, text_action,
};
use crate::install::page_records::{BoxRecord, Piece, piece_count};
use crate::install::vars::InstallerPages;
use rc2k_core::install::pieces::{self, Way};

fn font_of(index: u32) -> Result<crate::gfx::typefaces::FontId, BadInput> {
    crate::install::vars::InstallerPages::font_of(index)
}

pub fn measure_pieces() -> Result<(), Refused> {
    let count = piece_count();
    if count == 0 {
        return Err(BadInput("a page of no pieces").into());
    }
    // The size each piece gets is what ecx and edx hold: ecx the pieces left (the measures
    // get it as their x), edx the last height (a piece measured with neither keeps it).
    let mut height = None;
    for k in 0..count {
        let left = count - k;
        let piece = Piece::nth(k);
        // Converted: `rc2k_core::install::pieces` (`measured_kind`, `Way`,
        // `measure_colour_at`).
        let marks = piece.marks();
        if marks & 0x20 != 0 {
            piece.set_kind(pieces::measured_kind(marks, piece.kind()));
        }
        let (width, high) = match Way::of(marks, piece.kind()) {
            Way::String { font_kind } => {
                let font = InstallerPages::kind_font(pieces::measure_colour_at(font_kind)) as u32;
                measure_string(piece.text_id() as u32, font_of(font)?, left)?
            }
            Way::Edit => {
                let item = piece.item();
                let record = item.edit_field();
                let font_height =
                    InstallerPages::kind_font(7 + 8 * record.font_kind() as u32) as u32;
                let width = crate::install::folders::field_width(item.text(), record.width());
                (width as u32, font_height)
            }
            Way::Text => {
                let item = piece.item();
                let mut cursor = append_string_by_id(
                    piece.text_id() as u32,
                    &crate::strings::owner::strings().text.buffer.text,
                );
                let action = item.data();
                if action != 0 {
                    cursor = text_action(action, cursor)?;
                }
                append_byte(cursor, 0);
                let kind = item.font_kind() as u32;
                let font = InstallerPages::kind_font(pieces::measure_colour_at(kind)) as u32;
                measure_text(
                    font_of(font)?,
                    &crate::strings::owner::strings().text.buffer.text,
                    left,
                )?
            }
            Way::Action => {
                let the_box = BoxRecord::linked(piece.item().box_link());
                (the_box.width(), the_box.height())
            }
            Way::Other => {
                let Some(height) = height else {
                    return Err(BadInput(
                        "a first piece measured with nothing (the original's entry edx)",
                    )
                    .into());
                };
                (left, height)
            }
        };
        height = Some(high);
        piece.set_width(width);
        piece.set_height(high);
    }
    Ok(())
}
