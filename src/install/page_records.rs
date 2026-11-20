//! The installer's page records by name: the pieces of the page and the boxes they are placed in.
//!
//! - A piece (`Piece`): the pieces of the page being shown are `Install::pieces`, made by
//!   `install::lay_out_pieces`; each holds the item's kind and marks, its own kind, its text, its
//!   item and its place and size.
//! - A box (`BoxRecord`): `{x, y, width, height}`, an item's or the title's.

use crate::install::installer::refuse;
use crate::install::list_pages::BoxRect;
use crate::install::page_links::PageLink;
use crate::memory::list_handle::ListItem;

/// A piece of the page being shown: it keeps the list item it was made from.
pub type Piece = rc2k_core::install::layout::Piece<ListItem>;

/// The item a piece was made from; a piece made from none is refused.
pub fn item_of(piece: &Piece) -> ListItem {
    piece
        .item
        .unwrap_or_else(|| refuse(crate::errors::BadInput("a piece made from no item")))
}

/// A box `{x, y, width, height}` of dwords.
#[derive(Clone, Copy)]
pub struct BoxRecord(&'static BoxRect);

impl BoxRecord {
    /// The box a link names; none is the title's box.
    pub fn linked(install: &'static crate::install::owner::Install, link: PageLink) -> Self {
        if link.is_none() {
            return Self(&install.pages.boxes[1]);
        }
        Self(link.box_rect(install).unwrap_or_else(|e| refuse(e)))
    }

    pub fn x(self) -> u32 {
        self.0.x.get()
    }

    pub fn y(self) -> u32 {
        self.0.y.get()
    }

    pub fn width(self) -> u32 {
        self.0.width.get()
    }

    pub fn height(self) -> u32 {
        self.0.height.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::list_handle::ListMenu;

    /// A page laid out keeps each piece's item: the list's item (or the title) it was made from.
    #[test]
    fn a_piece_names_the_item_it_was_made_from() {
        let _lock = crate::test_support::test_lock();
        let install = crate::test_support::own::install();
        let saved = install.pieces.borrow().clone();
        for id in [
            crate::install::page_links::ListId::Main,
            crate::install::page_links::ListId::Room,
            crate::install::page_links::ListId::Folder,
        ] {
            let list = ListMenu::installer(install, id);
            crate::install::lay_out_pieces::lay_out_pieces(install, list).unwrap();
            let pieces = install.pieces.borrow().clone();
            assert!(pieces.len() > 1, "{id:?}");
            for (n, piece) in pieces.iter().enumerate() {
                let made_from = item_of(piece).installer_position(install).unwrap();
                assert!(
                    made_from == crate::install::list_pages::ItemPosition::Title
                        || matches!(made_from, crate::install::list_pages::ItemPosition::In(l, _) if l == id),
                    "{id:?} piece {n}: {made_from:?}"
                );
            }
        }
        *install.pieces.borrow_mut() = saved;
    }
}
