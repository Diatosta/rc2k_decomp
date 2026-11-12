//! `install::measure_pieces` (0x414DF5): each of a page's pieces measured into its width
//! (+0x10) and height (+0x14). A piece whose item has 0x20 first has bits 0, 1 of its kind
//! moved to bits 2, 3. A piece of a plain text kind (no 0x10) is its string measured
//! (`text::measure_string_by_id`) in the font of its marks and kind (`fonts`); else by the item's kind: an edit its record's width and its font's
//! height (`fonts`); a text its string and the item's appended text (`[item+0xC]`
//! given `[item+8]`, if any) in `text`, measured (`text::measure_text`); an action its
//! box's size; any other the size of the pieces before it.
//!
//! No pieces would go round 2^32 times: refused. Caller `install::run_page`.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{
    measure_string, measure_text, push_string_by_id, store_text, text_action,
};
use crate::install::page_records::{BoxRecord, Piece, piece_count};
use crate::install::vars::InstallerPages;
use crate::strings::owner::Strings;
use rc2k_core::install::pieces::{self, Way};

fn font_of(
    install: &crate::install::owner::Install,
    index: u32,
) -> Result<crate::gfx::typefaces::FontId, BadInput> {
    crate::install::vars::InstallerPages::font_of(install, index)
}

pub fn measure_pieces(
    install: &'static crate::install::owner::Install,
    strings: &'static Strings,
) -> Result<(), Refused> {
    let count = piece_count(install);
    if count == 0 {
        return Err(BadInput("a page of no pieces").into());
    }
    // The size each piece gets is the carried one: the pieces left (the measures get it as
    // their x) and the last height (a piece measured with neither keeps it).
    let mut height = None;
    for k in 0..count {
        let left = count - k;
        let piece = Piece::nth(install, k);
        // Converted: `rc2k_core::install::pieces` (`measured_kind`, `Way`,
        // `measure_colour_at`).
        let marks = piece.marks();
        if marks & 0x20 != 0 {
            piece.set_kind(pieces::measured_kind(marks, piece.kind()));
        }
        let (width, high) = match Way::of(marks, piece.kind()) {
            Way::String { font_kind } => {
                let font =
                    InstallerPages::kind_font(install, pieces::measure_colour_at(font_kind)) as u32;
                measure_string(piece.text_id() as u32, font_of(install, font)?, left)?
            }
            Way::Edit => {
                let item = piece.item();
                let record = item.edit_field();
                let font_height =
                    InstallerPages::kind_font(install, 7 + 8 * record.font_kind() as u32) as u32;
                let width =
                    crate::install::folders::field_width(install, item.text(), record.width());
                (width as u32, font_height)
            }
            Way::Text => {
                let item = piece.item();
                let mut text = Vec::new();
                push_string_by_id(&mut text, piece.text_id() as u32);
                let action = item.data();
                if action != 0 {
                    text_action(action, &mut text)?;
                }
                text.push(0);
                store_text(&strings.text.buffer.text, &text);
                let kind = item.font_kind() as u32;
                let font =
                    InstallerPages::kind_font(install, pieces::measure_colour_at(kind)) as u32;
                measure_text(font_of(install, font)?, &text, left)?
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
