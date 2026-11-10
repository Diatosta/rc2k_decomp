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

use crate::install::installer::{rb, rd, refuse, rl, rw, wb, wd, ww};
use crate::install::page_links::PageLink;
use crate::menu::list_handle::{ListItem, ListMenu};
use crate::place::BytePlace;
use crate::pointers::GamePtr;

/// A page entry (a place in the pages' tables).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageEntry(GamePtr);

impl PageEntry {
    /// The entry at a table's start.
    pub fn of(place: impl Into<BytePlace>) -> Self {
        Self(place.into().game_ptr())
    }

    /// The entry a link from another entry names.
    pub fn at(link: PageLink) -> Self {
        Self(link.in_pages().unwrap_or_else(|e| refuse(e)))
    }

    /// +0 as the link to the page's list (none: no list).
    fn list_link(self) -> PageLink {
        rl(self.0.limited(4))
    }

    /// Whether the entry has a list (+0).
    pub fn has_list(self) -> bool {
        !self.list_link().is_none()
    }

    /// The page's list.
    pub fn list(self) -> ListMenu {
        ListMenu::at(self.list_link()).unwrap_or_else(|e| refuse(e))
    }

    /// The entry `offset` bytes on (`pages::next_entry_at`) read as a link; None for none.
    pub fn next(self, offset: u32) -> Option<PageEntry> {
        let link = rl(self.0.offset(offset).limited(4));
        (!link.is_none()).then(|| PageEntry::at(link))
    }
}

/// The pieces' number (the layout's first dword).
pub fn piece_count(install: &'static crate::install::owner::Install) -> u32 {
    install.pages.layout.get_u32(0)
}

pub fn set_piece_count(install: &'static crate::install::owner::Install, count: u32) {
    wd(BytePlace::from(&install.pages.layout), count);
}

/// A piece of the layout.
#[derive(Clone, Copy)]
pub struct Piece(BytePlace);

/// A piece's bytes.
pub const PIECE: u32 = rc2k_core::install::layout::PIECE;

impl Piece {
    /// Piece `n` (wrapping as the original's sum); a piece outside the layout is refused
    /// when it is read or written.
    pub fn nth(n: u32) -> Self {
        let install = crate::install::owner::install();
        Self(BytePlace::from(&install.pages.layout).add(4 + PIECE.wrapping_mul(n)))
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

    /// +4: the piece's item (the original kept its address; this is its offset in the
    /// installer's pages, `ListItem::in_pages`).
    pub fn item(self) -> ListItem {
        ListItem::in_pages(rd(self.0.add(4))).unwrap_or_else(|e| refuse(e))
    }

    pub fn set_item(self, item: ListItem) {
        wd(
            self.0.add(4),
            item.pages_offset().unwrap_or_else(|e| refuse(e)),
        );
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
pub struct BoxRecord(BytePlace);

impl BoxRecord {
    /// The box a link names; none is the title's box.
    pub fn linked(link: PageLink) -> Self {
        let install = crate::install::owner::install();
        if link.is_none() {
            return Self(BytePlace::from(&install.pages.title_box));
        }
        Self(BytePlace::from(
            link.in_pages().unwrap_or_else(|e| refuse(e)),
        ))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every page of both runs resolves its list, its items' links and its neighbours table.
    #[test]
    fn every_page_resolves_its_links() {
        let mut seen = 0;
        for first in [
            PageEntry::of(&crate::install::owner::install().pages.normal_pages),
            PageEntry::of(&crate::install::owner::install().pages.update_pages),
        ] {
            let mut entries = vec![first];
            while let Some(entry) = entries.pop() {
                if entry.list_link().in_pages().is_err() || seen > 200 {
                    continue;
                }
                seen += 1;
                let list = entry.list();
                let _ = list.neighbours();
                for n in 0..u32::from(list.count()) {
                    let item = list.item(n);
                    let _ = item.box_link();
                    match item.kind() {
                        2 => {
                            let _ = item.choices().word(0);
                        }
                        4 => {
                            let _ = item.edit_field().width();
                            let _ = item.text();
                        }
                        _ => {}
                    }
                }
                // (An entry holds as many answers' links as its page takes; the rest of the
                // words are other data.)
                entries.extend((0..8).filter_map(|answer| {
                    let at = rc2k_core::install::pages::next_entry_at(answer);
                    let link = rl(entry.0.offset(at).limited(4));
                    link.in_pages().ok().map(|_| PageEntry::at(link))
                }));
            }
        }
        assert!(seen > 5);
    }
}
