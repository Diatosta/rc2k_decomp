//! The installer's page tables by name: the entries that chain the pages, the layout's pieces
//! and the boxes they are placed in.
//!
//! - A page entry (`PageEntry`): dwords in `InstallerPages::page_table`: +0 the page's list
//!   (`list_handle::ListMenu`), then the entries the page's answers lead on to
//!   (`rc2k_core::install::pages::next_entry_at`).
//! - A piece (`Piece`): the layout `InstallerPages::layout` holds a count (a dword) and 0x30
//!   bytes a piece, made by `install::lay_out_pieces`: +0 the item's kind and marks, +1 the
//!   piece's kind, +2 its text, +4 its item, +8 x, +0xC y, +0x10 width, +0x14 height.
//! - A box (`BoxRecord`): `{x, y, width, height}`, an item's (+4) or the title's.
//!
//! Reads and writes are checked: a place outside its buffer ends the program.

use crate::errors::BadInput;
use crate::install::installer::{rb, rd, refuse, rw, wb, wd, ww};
use crate::install::page_links::{PageLink, TABLE_LEN};
use crate::list_handle::{BoxRect, ListItem, ListMenu};
use crate::place::BytePlace;

/// The page table's index of the update run's first entry.
const UPDATE_RUN: u32 = 17;

/// A page entry: its index in the pages' table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageEntry(u32);

impl PageEntry {
    /// The entry's index in the pages' table.
    #[cfg(test)]
    pub fn index(self) -> u32 {
        self.0
    }

    pub fn normal_run() -> Self {
        Self(0)
    }

    pub fn update_run() -> Self {
        Self(UPDATE_RUN)
    }

    /// The entry a link from another entry names: a link to a table entry. The links of the
    /// pages' answers name entry starts (the census test below walks them all), so a link to
    /// anything else is refused.
    pub fn at(link: PageLink) -> Self {
        match link.table_index() {
            Some(index) => Self(index),
            None => refuse(BadInput("a page link that is no page entry")),
        }
    }

    /// +0 as the link to the page's list (none: no list).
    fn list_link(self) -> PageLink {
        Self::word(self.0)
    }

    pub fn has_list(self) -> bool {
        !self.list_link().is_none()
    }

    /// The page's list: the entry's first word is a link to one of the pages' lists.
    pub fn list(self) -> ListMenu {
        let link = self.list_link();
        if link.list_id().is_none() {
            refuse(BadInput("a page entry whose first word is no list"));
        }
        ListMenu::at(link).unwrap_or_else(|e| refuse(e))
    }

    /// How many answers the entry holds: the table words after its list's that link to an
    /// entry or to none, up to the next entry's list (the table's last word, 9, is no link).
    /// An answer past them is refused; no page leaves with such an answer
    /// (`answers_stay_inside_their_entry`).
    pub fn answers(self) -> u32 {
        let first = self.0;
        let mut end = first + 1;
        while end < TABLE_LEN
            && let word = Self::word(end)
            && (word.is_none() || word.table_index().is_some())
        {
            end += 1;
        }
        end - first - 1
    }

    /// The entry at byte `offset` of this one (`pages::next_entry_at`: 4 + 4 x answer),
    /// read as a link; None for none. An offset that is no answer of the entry is refused.
    pub fn next(self, offset: u32) -> Option<PageEntry> {
        if offset < 4 || !offset.is_multiple_of(4) || (offset - 4) / 4 >= self.answers() {
            refuse(BadInput("a page answer past the answers its entry holds"));
        }
        let link = Self::word(self.0 + offset / 4);
        (!link.is_none()).then(|| PageEntry::at(link))
    }

    /// Table word `index` as a link; an index off the table is refused.
    fn word(index: u32) -> PageLink {
        match crate::install::page_links::installer()
            .pages
            .page_table
            .get(index as usize)
        {
            Some(word) => PageLink::from_raw(word.get()),
            None => refuse(BadInput("a page table index past the table")),
        }
    }
}

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

    pub fn item(self) -> ListItem {
        ListItem::from_handle(rd(self.at(Field::Item))).unwrap_or_else(|e| refuse(e))
    }

    pub fn set_item(self, item: ListItem) {
        wd(
            self.at(Field::Item),
            item.to_handle().unwrap_or_else(|e| refuse(e)),
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
    pub fn linked(link: PageLink) -> Self {
        if link.is_none() {
            return Self(&crate::install::page_links::installer().pages.boxes[1]);
        }
        Self(link.box_rect().unwrap_or_else(|e| refuse(e)))
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
    use crate::install::page_links::ListId;

    #[test]
    fn every_page_resolves_its_links() {
        let mut seen = 0;
        for first in [PageEntry::normal_run(), PageEntry::update_run()] {
            let mut entries = vec![first];
            while let Some(entry) = entries.pop() {
                if entry.list_link().list_id().is_none() || seen > 200 {
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
                entries.extend((0..entry.answers()).filter_map(|answer| {
                    entry.next(rc2k_core::install::pages::next_entry_at(answer))
                }));
            }
        }
        assert!(seen > 5);
    }

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
            let list = ListMenu::installer(id);
            crate::install::lay_out_pieces::lay_out_pieces(install, list).unwrap();
            let pieces = piece_count(install);
            assert!(pieces > 1, "{id:?}");
            for n in 0..pieces {
                let item = Piece::nth(install, n).item();
                let made_from = item.installer_position().unwrap();
                assert!(
                    made_from == crate::list_handle::ItemPosition::Title
                        || matches!(made_from, crate::list_handle::ItemPosition::In(l, _) if l == id),
                    "{id:?} piece {n}: {made_from:?}"
                );
            }
        }
        for (n, &byte) in saved.iter().enumerate() {
            install.pages.layout.set(n, byte);
        }
    }

    /// M4 proof for `PageEntry::next`: every answer a page can leave with is a slot of its
    /// entry. A page leaves with 1 (the start value, the folder page's Escape, `ignore_room`),
    /// 0 (Escape, the folder page's Enter) or the next-list byte of a link item without an
    /// action (`item_link`; the finish page's 2 is made 1 by `finish_page`). The items'
    /// bytes are the pages' data: nothing writes them while an installer runs.
    #[test]
    fn answers_stay_inside_their_entry() {
        let mut checked = 0;
        for index in 0..crate::install::page_links::TABLE_LEN {
            let link = PageLink::table(index);
            let word = PageEntry::word(index);
            if word.list_id().is_none() {
                continue;
            }
            let entry = PageEntry::at(link);
            let list = entry.list();
            let mut answers = vec![0, 1];
            for n in 0..u32::from(list.count()) {
                let item = list.item(n);
                if item.kind() == 1 && item.target() == 0 && item.count() & 0x80 == 0 {
                    answers.push(u32::from(item.count()));
                }
            }
            if word.list_id() == Some(ListId::Finish) {
                answers.retain(|&a| a != 2);
            }
            let held = entry.answers();
            for answer in answers {
                assert!(answer < held, "entry {index}: answer {answer} of {held}");
                let _ = entry.next(rc2k_core::install::pages::next_entry_at(answer));
            }
            // Every item the page has is an item of its list.
            assert!(
                u32::from(list.count())
                    <= (match word.list_id() {
                        Some(ListId::Language | ListId::Drives8) => 10,
                        Some(ListId::Main | ListId::Mode) => 5,
                        Some(ListId::Drives16) => 18,
                        Some(ListId::DrivesMore) => 26,
                        Some(ListId::Folder) => 6,
                        Some(ListId::Room) => 3,
                        Some(ListId::Dxsetup | ListId::Finish) => 4,
                        _ => 0u32,
                    })
            );
            checked += 1;
        }
        assert_eq!(checked, 13);
    }
}
