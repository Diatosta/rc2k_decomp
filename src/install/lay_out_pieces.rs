//! `install::lay_out_pieces` (0x414C7E): a page's pieces (`layout` of them, 0x30 bytes
//! each after it: +0 the item's kind and marks, +1 the piece's kind, +2 its text, +4 its
//! item, +8 x, +0xC y, +0x10 width, +0x14 height) made from its list. The title (unless the
//! list's flags are 0x40 of 0xC0: `test; jg`) is kind 0x10 of `title`; then each item
//! not hidden (0x40 of 0xC0): a link a piece of kind 0; a choice a label (1) and a piece
//! (2) for each of its choices not hidden, its marks from the choice's high bits; a value
//! a label (1) and a piece (0x12); an edit the same, without the label when the item has
//! 0x10; a text kind 0x10 with its font's bits; an action 0x14. The places
//! and sizes are zeroed.
//!
//! Takes the list.
//! Any other kind is fatal error 0x80A97451. More pieces than the 0xFFC bytes hold (85) is
//! refused (the original writes over the list variables after them). Caller
//! `install::run_page`.

use crate::errors::{BadInput, Refused};
use crate::install::installer::fatal;
use crate::install::page_records::{Piece, set_piece_count};
use crate::menu::list_handle::{ListItem, ListMenu};
use rc2k_core::install::pieces::{self, Item, LayoutError, ListSource, Owner};

/// A page's list, read where the original reads it.
struct GameList(ListMenu);

impl ListSource for GameList {
    fn flags(&self) -> u8 {
        self.0.flags()
    }
    fn count(&self) -> u8 {
        self.0.count()
    }
    fn title_text(&self) -> u16 {
        self.0.title_text()
    }
    fn item(&self, k: u32) -> Item {
        let item = self.0.item(k);
        Item {
            first: item.first(),
            count: item.count(),
            text: item.text_id(),
        }
    }
    fn choice(&self, k: u32, c: u32) -> u16 {
        self.0.item(k).choices().word(c)
    }
    fn text_font(&self, font: u8) -> u8 {
        let install = crate::install::owner::install();
        install.pages.kind_fonts.get(4 + 8 * font as usize)
    }
}

pub fn lay_out_pieces(
    install: &'static crate::install::owner::Install,
    list: ListMenu,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::install::pieces::lay_out`; the pieces written here.
    let made = match pieces::lay_out(&GameList(list)) {
        Ok(made) => made,
        Err(LayoutError::UnknownKind) => return Err(fatal(pieces::UNKNOWN_KIND)),
        Err(LayoutError::TooMany) => {
            return Err(BadInput("more pieces than the page's table holds").into());
        }
    };
    for (n, new) in made.iter().enumerate() {
        let piece = Piece::nth(n as u32);
        if let Some(text) = new.text {
            piece.set_text_id(text);
        }
        piece.set_marks(new.marks);
        piece.set_kind(new.kind);
        piece.set_item(match new.owner {
            Owner::Title => ListItem::of_title(),
            Owner::Item(k) => list.item(k),
        });
        piece.set_x(0);
        piece.set_y(0);
        piece.set_width(0);
        piece.set_height(0);
    }
    set_piece_count(install, made.len() as u32);
    Ok(())
}
