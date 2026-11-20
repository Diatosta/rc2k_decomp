//! A page's pieces (0x30 bytes each after `layout`) made from its list: the title (unless the list
//! is hidden), then for each item not hidden: a link a piece, a choice a label and a piece for each
//! choice not hidden, a value or edit a label and a piece, a text and an action one piece each.
//! Places and sizes are zeroed. Any other kind is fatal error 0x80A97451. More than 85 pieces are
//! refused.

use crate::errors::{BadInput, Refused};
use crate::install::installer::fatal;
use crate::install::page_records::Piece;
use crate::memory::list_handle::{ListItem, ListMenu};
use rc2k_core::install::pieces::{self, Item, LayoutError, ListSource, Owner};

/// A page's list.
struct GameList(ListMenu, &'static crate::install::owner::Install);

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
        self.0.item(k).choices(self.1).word(c)
    }
    fn text_font(&self, font: u8) -> u8 {
        let install = self.1;
        install.pages.kind_fonts.get(4 + 8 * font as usize)
    }
}

pub(super) fn lay_out_pieces(
    install: &'static crate::install::owner::Install,
    list: ListMenu,
) -> Result<(), Refused> {
    let made = match pieces::lay_out(&GameList(list, install)) {
        Ok(made) => made,
        Err(LayoutError::UnknownKind) => return Err(fatal(pieces::UNKNOWN_KIND)),
        Err(LayoutError::TooMany) => {
            return Err(BadInput("more pieces than the page's table holds").into());
        }
    };
    let laid = made
        .iter()
        .map(|new| Piece {
            marks: new.marks,
            kind: new.kind,
            // A piece made with no text keeps the table's word it had.
            text: new.text.unwrap_or(0),
            item: Some(match new.owner {
                Owner::Title => ListItem::of_title(install),
                Owner::Item(k) => list.item(k),
            }),
            ..Piece::default()
        })
        .collect();
    *install.pieces.borrow_mut() = laid;
    Ok(())
}
