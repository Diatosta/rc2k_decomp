//! Each of a page's pieces measured into its width and height: a plain text piece as its string in
//! the font of its marks; else by the item's kind: an edit its record's width and height, a text
//! its string and appended text, an action its box's size, any other the size of the pieces before
//! it. Refused: no pieces.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{
    measure_string, measure_text, push_string_by_id, store_text, text_action,
};
use crate::install::page_records::{BoxRecord, item_of};
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
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    install: &'static crate::install::owner::Install,
    strings: &'static Strings,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    let mut laid = install.pieces.borrow().clone();
    let count = laid.len() as u32;
    if count == 0 {
        return Err(BadInput("a page of no pieces").into());
    }
    // The size each piece gets is the carried one: the pieces left (the measures get it as
    // their x) and the last height (a piece measured with neither keeps it).
    let mut height = None;
    for (k, piece) in (0..).zip(laid.iter_mut()) {
        let left = count - k;
        let marks = piece.marks;
        if marks & 0x20 != 0 {
            piece.kind = pieces::measured_kind(marks, piece.kind);
        }
        let (width, high) = match Way::of(marks, piece.kind) {
            Way::String { font_kind } => {
                let font =
                    InstallerPages::kind_font(install, pieces::measure_colour_at(font_kind)) as u32;
                measure_string(so, piece.text as u32, font_of(install, font)?, left, gfx)?
            }
            Way::Edit => {
                let item = item_of(piece);
                let record = item.edit_field(install);
                let font_height =
                    InstallerPages::kind_font(install, 7 + 8 * record.font_kind() as u32) as u32;
                let width = crate::install::folders::field_width(
                    install,
                    item.text(install),
                    record.width(),
                );
                (width as u32, font_height)
            }
            Way::Text => {
                let item = item_of(piece);
                let mut text = Vec::new();
                push_string_by_id(so, &mut text, piece.text as u32);
                let action = item.data();
                if action != 0 {
                    text_action(action, &mut text)?;
                }
                text.push(0);
                store_text(&strings.text.buffer.text, &text);
                let kind = item.font_kind() as u32;
                let font =
                    InstallerPages::kind_font(install, pieces::measure_colour_at(kind)) as u32;
                measure_text(font_of(install, font)?, &text, left, gfx)?
            }
            Way::Action => {
                let the_box = BoxRecord::linked(install, item_of(piece).box_link());
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
        piece.width = width;
        piece.height = high;
    }
    *install.pieces.borrow_mut() = laid;
    Ok(())
}
