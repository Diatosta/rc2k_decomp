//! A page's pieces (0x30 bytes each after `layout`) made from its list: the title (unless the
//! list is hidden), then for each item not hidden: a link a piece, a choice a label and a piece
//! for each choice not hidden, a value or edit a label and a piece, a text and an action one
//! piece each. Places and sizes are zeroed. Any other kind is fatal error 0x80A97451. More
//! pieces than the 0xFFC bytes hold (85) are refused. Called by `install::run_page`.

use crate::errors::{BadInput, Refused};
use crate::install::installer::fatal;
use crate::install::page_records::{Piece, set_piece_count};
use crate::menu::list_handle::{ListItem, ListMenu};
use rc2k_core::install::pieces::{self, Item, LayoutError, ListSource, Owner};

/// A page's list.
struct GameList<'a>(ListMenu, &'a crate::install::owner::Install);

impl ListSource for GameList<'_> {
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
        let install = self.1;
        install.pages.kind_fonts.get(4 + 8 * font as usize)
    }
}

pub fn lay_out_pieces(
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
    for (n, new) in made.iter().enumerate() {
        let piece = Piece::nth(install, n as u32);
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
