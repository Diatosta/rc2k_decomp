//! Each of a page's pieces measured into its width and height: a plain text piece as its string in
//! the font of its marks; else by the item's kind: an edit its record's width and height, a text
//! its string and appended text, an action its box's size, any other the size of the pieces before
//! it. Refused: no pieces.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{measure_string, measure_text, push_string_by_id, store_text};
use crate::install::lay_out_pieces::{Piece, item_of};
use crate::install::vars::InstallerPages;
use crate::strings::owner::Strings;
use rc2k_core::install::model::{Kind, PageDef};
use rc2k_core::install::pieces::{self, Way};

fn font_of(
    install: &crate::install::owner::Install,
    index: u32,
) -> Result<crate::gfx::typefaces::FontId, BadInput> {
    crate::install::vars::InstallerPages::font_of(install, index)
}

pub(super) fn measure_pieces(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    install: &'static crate::install::owner::Install,
    def: &PageDef,
    laid: &mut [Piece],
    strings: &'static Strings,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
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
                let Kind::Edit(edit) = &item_of(def, piece)?.kind else {
                    return Err(BadInput("an edit piece of an item that is no edit").into());
                };
                let font_height =
                    InstallerPages::kind_font(install, 7 + 8 * u32::from(edit.font_kind)) as u32;
                (u32::from(edit.width), font_height)
            }
            Way::Text => {
                let Kind::Text {
                    font_kind,
                    ends_line,
                } = item_of(def, piece)?.kind
                else {
                    return Err(BadInput("a text piece of an item that is no text").into());
                };
                let mut text = Vec::new();
                push_string_by_id(so, &mut text, piece.text as u32);
                if ends_line {
                    crate::install::end_text_line::end_text_line(&mut text);
                }
                text.push(0);
                store_text(&strings.text.buffer.text, &text);
                let font =
                    InstallerPages::kind_font(install, pieces::measure_colour_at(font_kind as u32))
                        as u32;
                measure_text(font_of(install, font)?, &text, left, gfx)?
            }
            Way::Action => return Err(BadInput("an action item (the pages hold none)").into()),
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
    Ok(())
}
