//! The installer's page records by name: the layout's pieces and the boxes they are placed in.
//!
//! - A piece (`Piece`): the layout `InstallerPages::layout` holds a count (a dword) and 0x30
//!   bytes a piece, made by `install::lay_out_pieces`: +0 the item's kind and marks, +1 the
//!   piece's kind, +2 its text, +4 its item, +8 x, +0xC y, +0x10 width, +0x14 height.
//! - A box (`BoxRecord`): `{x, y, width, height}`, an item's (+4) or the title's.
//!
//! Reads and writes are checked: a place outside its buffer ends the program.

use crate::install::installer::{rb, rd, refuse, rw, wb, wd, ww};
use crate::install::page_links::PageLink;
use crate::memory::list_handle::{BoxRect, ListItem};
use crate::memory::place::BytePlace;

/// The pieces' number (the layout's first dword).
pub fn piece_count(install: &'static crate::install::owner::Install) -> u32 {
    install.pages.layout.get_u32(0)
}

pub fn set_piece_count(install: &'static crate::install::owner::Install, count: u32) {
    wd(BytePlace::from(&install.pages.layout), count);
}

#[derive(Clone, Copy)]
pub struct Piece(BytePlace);

/// A piece's bytes.
pub const PIECE: u32 = rc2k_core::install::layout::PIECE;

/// The fields of a piece, each at its byte in the piece's 0x30 bytes.
#[derive(Clone, Copy)]
enum Field {
    /// The item's kind and marks.
    Marks = 0,
    /// The piece's kind.
    Kind = 1,
    /// The piece's text (a string id).
    Text = 2,
    /// The piece's item, as the handle `ListItem::to_handle` makes.
    Item = 4,
    X = 8,
    Y = 0xC,
    Width = 0x10,
    Height = 0x14,
}

impl Piece {
    /// Piece `n` (wrapping); a piece outside the layout is refused when it is read or written.
    pub fn nth(install: &'static crate::install::owner::Install, n: u32) -> Self {
        Self(BytePlace::from(&install.pages.layout).add(4 + PIECE.wrapping_mul(n)))
    }

    fn at(self, field: Field) -> BytePlace {
        self.0.add(field as u32)
    }

    pub fn marks(self) -> u8 {
        rb(self.at(Field::Marks))
    }

    pub fn set_marks(self, marks: u8) {
        wb(self.at(Field::Marks), marks);
    }

    pub fn kind(self) -> u8 {
        rb(self.at(Field::Kind))
    }

    pub fn set_kind(self, kind: u8) {
        wb(self.at(Field::Kind), kind);
    }

    pub fn text_id(self) -> u16 {
        rw(self.at(Field::Text))
    }

    pub fn set_text_id(self, text: u16) {
        ww(self.at(Field::Text), text);
    }

    pub fn item(self, install: &'static crate::install::owner::Install) -> ListItem {
        ListItem::from_handle(install, rd(self.at(Field::Item))).unwrap_or_else(|e| refuse(e))
    }

    pub fn set_item(self, install: &'static crate::install::owner::Install, item: ListItem) {
        wd(
            self.at(Field::Item),
            item.to_handle(install).unwrap_or_else(|e| refuse(e)),
        );
    }

    pub fn x(self) -> u32 {
        rd(self.at(Field::X))
    }

    pub fn set_x(self, x: u32) {
        wd(self.at(Field::X), x);
    }

    pub fn y(self) -> u32 {
        rd(self.at(Field::Y))
    }

    pub fn set_y(self, y: u32) {
        wd(self.at(Field::Y), y);
    }

    pub fn width(self) -> u32 {
        rd(self.at(Field::Width))
    }

    pub fn set_width(self, width: u32) {
        wd(self.at(Field::Width), width);
    }

    pub fn height(self) -> u32 {
        rd(self.at(Field::Height))
    }

    pub fn set_height(self, height: u32) {
        wd(self.at(Field::Height), height);
    }
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

    /// A page laid out keeps each piece's item as a handle that names the list's item (or the
    /// title) it was made from.
    #[test]
    fn a_piece_names_the_item_it_was_made_from() {
        let _lock = crate::test_support::test_lock();
        let install = crate::test_support::own::install();
        let saved: Vec<u8> = (0..4096).map(|n| install.pages.layout.get(n)).collect();
        for id in [
            crate::install::page_links::ListId::Main,
            crate::install::page_links::ListId::Room,
            crate::install::page_links::ListId::Folder,
        ] {
            let list = ListMenu::installer(install, id);
            crate::install::lay_out_pieces::lay_out_pieces(install, list).unwrap();
            let pieces = piece_count(install);
            assert!(pieces > 1, "{id:?}");
            for n in 0..pieces {
                let item = Piece::nth(install, n).item(install);
                let made_from = item.installer_position(install).unwrap();
                assert!(
                    made_from == crate::memory::list_handle::ItemPosition::Title
                        || matches!(made_from, crate::memory::list_handle::ItemPosition::In(l, _) if l == id),
                    "{id:?} piece {n}: {made_from:?}"
                );
            }
        }
        for (n, &byte) in saved.iter().enumerate() {
            install.pages.layout.set(n, byte);
        }
    }
}
