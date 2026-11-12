//! The installer's page tables by name: the entries that chain the pages, the layout's
//! pieces and the boxes they are placed in, in place of sums on game addresses.
//!
//! - A page entry (`PageEntry`): dwords in `InstallerPages::page_table`: +0 the page's list (`menu::list_handle::ListMenu`), and from
//!   there the entries the page's answers lead on to (`rc2k_core::install::pages::
//!   next_entry_at`).
//! - A piece (`Piece`): the layout `InstallerPages::layout` holds a count (a dword) and
//!   0x30 bytes a piece, made by `install::lay_out_pieces`: +0 the item's kind and marks,
//!   +1 the piece's kind, +2 its text, +4 its item, +8 x, +0xC y, +0x10 width, +0x14 height.
//! - A box (`BoxRecord`): four dwords `{x, y, width, height}`, an item's (+4) or the title's.
//!
//! Reads and writes are checked as `install::installer::{rb, ..}` are: a place outside its
//! buffer or every allocation ends the program.

use crate::errors::BadInput;
use crate::install::installer::{rb, rd, refuse, rw, wb, wd, ww};
use crate::install::page_links::PageLink;
use crate::install::pages_map::PagesAt;
use crate::menu::list_handle::{ListItem, ListMenu};
use crate::place::BytePlace;

/// The page table's index of the update run's first entry (0x51D5D8, from 0x51D594).
const UPDATE_RUN: u32 = 17;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// A page entry: its byte offset in the installer pages.
pub struct PageEntry(PagesAt);

impl PageEntry {
    /// The first entry of a normal run.
    pub fn normal_run() -> Self {
        Self::at(PageLink::table(0))
    }

    /// The first entry of an update run.
    pub fn update_run() -> Self {
        Self::at(PageLink::table(UPDATE_RUN))
    }

    /// The entry a link from another entry names. An entry's words past its answers are other
    /// data of the pages (the next entry's list, a list's bytes), which the page walk may
    /// read as links (`run_pages::pass_over` asks the drive page's eight): any thing of the
    /// pages is taken, and read as the original reads it.
    pub fn at(link: PageLink) -> Self {
        Self(link.pages_at().unwrap_or_else(|| {
            refuse(BadInput(
                "a page link that is no place in the installer's pages",
            ))
        }))
    }

    /// +0 as the link to the page's list (none: no list).
    fn list_link(self) -> PageLink {
        Self::word(self.0)
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
        let link = Self::word(self.0.plus_wide(offset));
        (!link.is_none()).then(|| PageEntry::at(link))
    }

    /// The link at byte `at` of the pages.
    fn word(at: PagesAt) -> PageLink {
        PageLink::from_raw(at.dword().unwrap_or_else(|e| refuse(e)))
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

/// The fields of a piece, each at its byte in the piece's 0x30 bytes.
#[derive(Clone, Copy)]
enum Field {
    /// The item's kind and marks.
    Marks = 0,
    /// The piece's kind.
    Kind = 1,
    /// The piece's text (a string id).
    Text = 2,
    /// The piece's item (the original kept its address; this is its offset in the
    /// installer's pages, `ListItem::in_pages`).
    Item = 4,
    X = 8,
    Y = 0xC,
    Width = 0x10,
    Height = 0x14,
}

impl Piece {
    /// Piece `n` (wrapping as the original's sum); a piece outside the layout is refused
    /// when it is read or written.
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

    pub fn item(self) -> ListItem {
        ListItem::in_pages(rd(self.at(Field::Item))).unwrap_or_else(|e| refuse(e))
    }

    pub fn set_item(self, item: ListItem) {
        wd(
            self.at(Field::Item),
            item.pages_offset().unwrap_or_else(|e| refuse(e)),
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
pub struct BoxRecord(PagesAt);

impl BoxRecord {
    /// The box a link names; none is the title's box.
    pub fn linked(link: PageLink) -> Self {
        if link.is_none() {
            return Self(PagesAt::box_at(1));
        }
        Self(link.box_position().unwrap_or_else(|e| refuse(e)))
    }

    fn word(self, at: u32) -> u32 {
        self.0.plus_wide(at).dword().unwrap_or_else(|e| refuse(e))
    }

    pub fn x(self) -> u32 {
        self.word(0)
    }

    pub fn y(self) -> u32 {
        self.word(4)
    }

    pub fn width(self) -> u32 {
        self.word(8)
    }

    pub fn height(self) -> u32 {
        self.word(0xC)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every page of both runs resolves its list, its items' links and its neighbours table.
    #[test]
    fn every_page_resolves_its_links() {
        let mut seen = 0;
        for first in [PageEntry::normal_run(), PageEntry::update_run()] {
            let mut entries = vec![first];
            while let Some(entry) = entries.pop() {
                if entry.list_link().pages_at().is_none() || seen > 200 {
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
                    let link = PageEntry::word(entry.0.plus_wide(at));
                    link.pages_at().map(|_| PageEntry::at(link))
                }));
            }
        }
        assert!(seen > 5);
    }
}
