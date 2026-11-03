//! The installer's page tables by name: the entries that chain the pages, the layout's
//! pieces and the boxes they are placed in, in place of sums on game addresses.
//!
//! - A page entry (`PageEntry`): dwords in `InstallerPages::{normal_pages, update_pages}`
//!   and their neighbours: +0 the page's list (`menu::list_handle::ListMenu`), and from
//!   there the entries the page's answers lead on to (`rc2k_core::install::pages::
//!   next_entry_at`).
//! - A piece (`Piece`): the layout `InstallerPages::layout` holds a count (a dword) and
//!   0x30 bytes a piece, made by `install::lay_out_pieces`: +0 the item's kind and marks,
//!   +1 the piece's kind, +2 its text, +4 its item, +8 x, +0xC y, +0x10 width, +0x14 height.
//! - A box (`BoxRecord`): four dwords `{x, y, width, height}`, an item's (+4) or the title's.
//!
//! Reads and writes are checked as `install::installer::{rb, ..}` are: a place outside its
//! buffer or every allocation ends the program.

use crate::install::installer::{rb, rd, rw, wb, wd, ww};
use crate::install::vars::INSTALLER_PAGES;
use crate::menu::list_handle::{ListItem, ListMenu};
use crate::place::Place;
use crate::pointers::GamePtr;

/// A page entry (a game address in the pages' tables).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageEntry(GamePtr);

impl PageEntry {
    /// The entry at a table's start.
    pub fn of(place: impl Into<Place>) -> Self {
        Self(place.into().game_ptr())
    }

    /// The entry at a game address (a link from another entry).
    pub fn at(addr: u32) -> Self {
        Self(GamePtr::new(addr))
    }

    /// +0: the page's list (0: none).
    pub fn list_addr(self) -> u32 {
        rd(self.0.limited(4))
    }

    /// The page's list.
    pub fn list(self) -> ListMenu {
        ListMenu::at(self.list_addr())
    }

    /// The entry `offset` bytes on (`pages::next_entry_at`) read as a link; None for 0.
    pub fn next(self, offset: u32) -> Option<PageEntry> {
        let at = rd(self.0.offset(offset).limited(4));
        (at != 0).then(|| PageEntry::at(at))
    }
}

/// The pieces' number (the layout's first dword).
pub fn piece_count() -> u32 {
    INSTALLER_PAGES.layout.get_u32(0)
}

pub fn set_piece_count(count: u32) {
    wd(Place::from(&INSTALLER_PAGES.layout), count);
}

/// A piece of the layout.
#[derive(Clone, Copy)]
pub struct Piece(Place);

/// A piece's bytes.
pub const PIECE: u32 = rc2k_core::install::layout::PIECE;

impl Piece {
    /// Piece `n` (wrapping as the original's sum); a piece outside the layout is refused
    /// when it is read or written.
    pub fn nth(n: u32) -> Self {
        Self(Place::from(&INSTALLER_PAGES.layout).add(4 + PIECE.wrapping_mul(n)))
    }

    /// +0: the item's kind and marks.
    pub fn marks(self) -> u8 {
        rb(self.0)
    }

    pub fn set_marks(self, marks: u8) {
        wb(self.0, marks);
    }

    /// +1: the piece's kind.
    pub fn kind(self) -> u8 {
        rb(self.0.add(1))
    }

    pub fn set_kind(self, kind: u8) {
        wb(self.0.add(1), kind);
    }

    /// +2: the piece's text (a string id).
    pub fn text_id(self) -> u16 {
        rw(self.0.add(2))
    }

    pub fn set_text_id(self, text: u16) {
        ww(self.0.add(2), text);
    }

    /// +4: the piece's item.
    pub fn item(self) -> ListItem {
        ListItem::at(self.item_addr())
    }

    pub fn item_addr(self) -> u32 {
        rd(self.0.add(4))
    }

    pub fn set_item(self, item: u32) {
        wd(self.0.add(4), item);
    }

    pub fn x(self) -> u32 {
        rd(self.0.add(8))
    }

    pub fn set_x(self, x: u32) {
        wd(self.0.add(8), x);
    }

    pub fn y(self) -> u32 {
        rd(self.0.add(0xC))
    }

    pub fn set_y(self, y: u32) {
        wd(self.0.add(0xC), y);
    }

    pub fn width(self) -> u32 {
        rd(self.0.add(0x10))
    }

    pub fn set_width(self, width: u32) {
        wd(self.0.add(0x10), width);
    }

    pub fn height(self) -> u32 {
        rd(self.0.add(0x14))
    }

    pub fn set_height(self, height: u32) {
        wd(self.0.add(0x14), height);
    }
}

/// A box `{x, y, width, height}` of dwords.
#[derive(Clone, Copy)]
pub struct BoxRecord(Place);

impl BoxRecord {
    pub fn of(place: impl Into<Place>) -> Self {
        Self(place.into())
    }

    pub fn x(self) -> u32 {
        rd(self.0)
    }

    pub fn y(self) -> u32 {
        rd(self.0.add(4))
    }

    pub fn width(self) -> u32 {
        rd(self.0.add(8))
    }

    pub fn height(self) -> u32 {
        rd(self.0.add(0xC))
    }
}
