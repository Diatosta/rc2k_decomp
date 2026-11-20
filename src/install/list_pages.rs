//! The installer's lists: the typed records behind the list menu (`memory::list_handle`)
//! the installer's pages hold, and the accessors that resolve their links.
//!
//! The installer's lists are typed records (`ListData`, `Item`, `EditRecord`) in
//! `install::vars::InstallerPages`, one per `ListId`; their words that name data (boxes,
//! texts, edit records) are links (`install::page_links::PageLink`) each accessor resolves by
//! kind, and the words that name code stay as read. An item the list does not hold is refused.

use crate::install::installer::refuse;
use crate::install::installer::{rb, rw, wb};
use crate::install::page_links::{ListId, PageLink};
use crate::memory::list_handle::{Item, ListHeader, ListItem, ListMenu};
use crate::memory::place::BytePlace;
use crate::memory::session::{PlainByte, PlainDword, PlainWord};

/// An edit item's record as the installer's pages hold it (8 bytes; `EditField` reads it).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct EditRecord {
    pub flags: PlainByte,
    /// +1: the most characters.
    pub most: PlainByte,
    /// +2: the default text.
    pub default_text: PlainWord,
    /// +4: the font's kind.
    pub font_kind: PlainByte,
    /// +5: nothing reads it.
    pub unused_05: PlainByte,
    /// +6: the width.
    pub width: PlainWord,
}

impl EditRecord {
    pub const fn new(
        flags: u8,
        most: u8,
        default_text: u16,
        font_kind: u8,
        unused_05: u8,
        width: u16,
    ) -> Self {
        Self {
            flags: PlainByte::new(flags),
            most: PlainByte::new(most),
            default_text: PlainWord::new(default_text),
            font_kind: PlainByte::new(font_kind),
            unused_05: PlainByte::new(unused_05),
            width: PlainWord::new(width),
        }
    }
}

/// A box `{x, y, width, height}` of dwords as the installer's pages hold it.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct BoxRect {
    pub x: PlainDword,
    pub y: PlainDword,
    pub width: PlainDword,
    pub height: PlainDword,
}

impl BoxRect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x: PlainDword::new(x),
            y: PlainDword::new(y),
            width: PlainDword::new(width),
            height: PlainDword::new(height),
        }
    }
}

/// The header and items of one of the installer's lists.
fn list_parts(
    install: &'static crate::install::owner::Install,
    list: ListId,
) -> (&'static ListHeader, &'static [Item]) {
    let pages = &install.pages;
    match list {
        ListId::Language => (&pages.language_menu.header, &pages.language_menu.items),
        ListId::Main => (&pages.main_menu.header, &pages.main_menu.items),
        ListId::Drives8 => (&pages.drive_menu_8.header, &pages.drive_menu_8.items),
        ListId::Drives16 => (&pages.drive_menu_16.header, &pages.drive_menu_16.items),
        ListId::DrivesMore => (&pages.menu_more.header, &pages.menu_more.items),
        ListId::Folder => (&pages.folder_menu.header, &pages.folder_menu.items),
        ListId::Room => (&pages.room_menu.header, &pages.room_menu.items),
        ListId::Removal => (&pages.removal_menu.header, &pages.removal_menu.items),
    }
}

#[cfg(test)]
/// The typed items of one of the installer's lists.
pub(crate) fn list_items(
    install: &'static crate::install::owner::Install,
    list: ListId,
) -> &'static [Item] {
    list_parts(install, list).1
}

impl ListMenu {
    /// One of the installer's lists.
    pub fn installer(install: &'static crate::install::owner::Install, list: ListId) -> Self {
        let (header, items) = list_parts(install, list);
        Self { header, items }
    }

    /// +0x10: the neighbours table for `menu::list_step` (None for none). No list holds one
    /// that a link could name: a list that has one is refused.
    pub fn neighbours(self) -> Option<BytePlace> {
        let link = PageLink::from_raw(self.header.neighbours.get());
        (!link.is_none()).then(|| {
            refuse(crate::errors::BadInput(
                "a list with a neighbours table the pages do not hold",
            ))
        })
    }
}

#[cfg(test)]
/// Which typed item of the installer's pages an item is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ItemPosition {
    /// The title's item.
    Title,
    /// Item `.1` of list `.0`.
    In(ListId, u32),
}

impl ListItem {
    #[cfg(test)]
    /// Where `item`, one of the pages' typed items, is.
    fn position(
        install: &'static crate::install::owner::Install,
        item: &Item,
    ) -> Option<ItemPosition> {
        if core::ptr::eq(item, &raw const install.pages.title) {
            return Some(ItemPosition::Title);
        }
        ListId::ALL.into_iter().find_map(|list| {
            list_items(install, list)
                .iter()
                .position(|typed| core::ptr::eq(typed, item))
                .map(|n| ItemPosition::In(list, n as u32))
        })
    }

    #[cfg(test)]
    /// Which typed item of the installer's pages this is (none for the viewer's items).
    pub(super) fn installer_position(
        self,
        install: &'static crate::install::owner::Install,
    ) -> Option<ItemPosition> {
        Self::position(install, self.0)
    }

    /// The title's item (the pages' `title` record).
    pub(super) fn of_title(install: &'static crate::install::owner::Install) -> Self {
        Self(&install.pages.title)
    }

    /// +4: the box the item's pieces are placed in (none: the title's).
    pub fn box_link(self) -> PageLink {
        PageLink::from_raw(self.0.box_link.get())
    }

    /// +8 as the link.
    fn target_link(self) -> PageLink {
        PageLink::from_raw(self.target())
    }

    /// +8 as the text the item edits or shows (a link).
    pub fn text(self, install: &'static crate::install::owner::Install) -> BytePlace {
        self.link_place(install)
    }

    /// +8 as the byte the item sets (a choice, a value; a link).
    pub fn value(self, install: &'static crate::install::owner::Install) -> ItemByte {
        ItemByte(self.link_place(install))
    }

    /// The place the link at +8 names: the folder typed.
    fn link_place(self, install: &'static crate::install::owner::Install) -> BytePlace {
        self.target_link()
            .place(install)
            .unwrap_or_else(|e| refuse(e))
    }

    /// +0xC as the item's choices (a choice item; a link): the table is the place the link
    /// names (`PageLink::place`), read on past the table (see `Choices`).
    pub fn choices(self, install: &'static crate::install::owner::Install) -> Choices {
        Choices(
            PageLink::from_raw(self.data())
                .place(install)
                .unwrap_or_else(|e| refuse(e)),
        )
    }

    /// +0xC as the item's edit record (an edit item).
    pub(super) fn edit_field(self, install: &'static crate::install::owner::Install) -> EditField {
        EditField(
            PageLink::from_raw(self.data())
                .edit_record(install)
                .unwrap_or_else(|e| refuse(e)),
        )
    }
}

/// The byte an item sets (its choice or its value).
#[derive(Clone, Copy, Debug)]
pub struct ItemByte(BytePlace);

impl ItemByte {
    pub fn get(self) -> u8 {
        rb(self.0)
    }

    pub fn set(self, value: u8) {
        wb(self.0, value);
    }
}

/// A choice item's table: two bytes a choice, the second (at +1, +3, ...) its marks. Its
/// length is the item's own count, which an index is not checked against: a read past it is
/// bounded by the place's buffer.
#[derive(Clone, Copy, Debug)]
pub struct Choices(BytePlace);

impl Choices {
    /// Where choice `n`'s marks byte is.
    fn marks_at(self, n: u32) -> BytePlace {
        self.0.add(n.wrapping_mul(2).wrapping_add(1))
    }

    /// Choice `n`'s word: its string, its high bits marks.
    pub fn word(self, n: u32) -> u16 {
        rw(self.0.add(n.wrapping_mul(2)))
    }

    /// Choice `n`'s byte at +1.
    pub fn marks(self, n: u32) -> u8 {
        rb(self.marks_at(n))
    }

    pub fn set_marks(self, n: u32, marks: u8) {
        wb(self.marks_at(n), marks);
    }

    /// The table's first `len` bytes (a `len` the item's count gives).
    pub fn bytes(self, len: usize, what: &str) -> Result<Vec<u8>, crate::errors::BadInput> {
        self.0.read(len, what)
    }

    pub fn write_bytes(self, bytes: &[u8], what: &str) -> Result<(), crate::errors::BadInput> {
        self.0.write(bytes, what)
    }
}

/// An edit item's record: the flags, the most characters, the default text, the font's kind
/// and the width.
#[derive(Clone, Copy)]
pub struct EditField(&'static EditRecord);

impl core::fmt::Debug for EditField {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EditField(the folder's)")
    }
}

impl EditField {
    pub fn flags(self) -> u8 {
        self.0.flags.get()
    }

    pub(super) fn set_flags(self, flags: u8) {
        self.0.flags.set(flags);
    }

    pub fn most(self) -> u8 {
        self.0.most.get()
    }

    pub(super) fn default_text(self) -> u16 {
        self.0.default_text.get()
    }

    pub fn font_kind(self) -> u8 {
        self.0.font_kind.get()
    }

    pub fn width(self) -> u16 {
        self.0.width.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::AssertUnwindSafe;

    /// The most items each installer list's record holds.
    fn held(list: ListId) -> u32 {
        match list {
            ListId::Language | ListId::Drives8 => 10,
            ListId::Main => 5,
            ListId::Drives16 => 18,
            ListId::DrivesMore => 26,
            ListId::Folder => 6,
            ListId::Room => 3,
            ListId::Removal => 4,
        }
    }

    #[test]
    fn an_installer_item_knows_its_place() {
        let install = crate::test_support::own::install();
        let _lock = crate::test_support::test_lock();
        assert_eq!(
            ListItem::of_title(install).installer_position(install),
            Some(ItemPosition::Title)
        );
        for list in ListId::ALL {
            for n in 0..held(list) {
                let item = ListMenu::installer(install, list).item(n);
                assert_eq!(
                    item.installer_position(install),
                    Some(ItemPosition::In(list, n))
                );
            }
        }
    }

    /// M4 for the installer's lists: every list's count starts within the items its record
    /// holds, so no page asks for an item past them, which is refused. The drive lists' counts
    /// are set by `list_drives` (the drives and two more, at most 24 drives: `drives::kept`).
    #[test]
    fn the_counts_stay_within_the_items() {
        let install = crate::test_support::own::install();
        let _lock = crate::test_support::test_lock();
        for list in ListId::ALL {
            let menu = ListMenu::installer(install, list);
            assert!(u32::from(menu.count()) <= held(list), "{list:?}");
            assert_eq!(
                list_items(install, list).len() as u32,
                held(list),
                "{list:?}"
            );
        }
    }

    /// M4 for the choice items: no item of the installer's lists is a choice item (kind 2),
    /// and the only run-time write of an item's kind is the main page's first item, made
    /// kind 5 (`check_at_start::set_script_entry`). The viewer's items are kind 1 or 5 in
    /// their starting data and only their marks (the high bits) change. The pages hold no
    /// table a choice link could name either: only the typed folder's link is a place.
    #[test]
    fn the_lists_hold_no_choice_items() {
        let install = crate::test_support::own::install();
        let _lock = crate::test_support::test_lock();
        for list in ListId::ALL {
            let menu = ListMenu::installer(install, list);
            for n in 0..u32::from(menu.count()) {
                assert_ne!(menu.item(n).kind(), 2, "{list:?} item {n}");
            }
        }
        let viewer = ListMenu::viewer(
            &crate::test_support::own::race()
                .replay
                .replay_view()
                .viewer_list,
        );
        for n in 0..u32::from(viewer.count()) {
            assert_ne!(viewer.item(n).kind(), 2, "viewer item {n}");
        }
    }

    #[test]
    fn a_choice_link_names_a_place_or_is_refused() {
        let install = crate::test_support::own::install();
        let _lock = crate::test_support::test_lock();
        let room = &crate::test_support::own::install().pages.room_menu.items[0];
        let saved = (room.first.get(), room.data.get());
        room.first.set(2);
        for link in [
            PageLink::box_at(8),
            PageLink::folder_edit(),
            PageLink::from_raw(0),
        ] {
            room.data.set(link.raw());
            let item = ListMenu::installer(install, ListId::Room).item(0);
            assert!(
                std::panic::catch_unwind(AssertUnwindSafe(|| item.choices(install))).is_err(),
                "{link:?}"
            );
        }
        room.data.set(PageLink::typed(2).raw());
        let item = ListMenu::installer(install, ListId::Room).item(0);
        assert!(
            std::panic::catch_unwind(AssertUnwindSafe(|| item.choices(install).marks(0))).is_ok()
        );
        room.first.set(saved.0);
        room.data.set(saved.1);
    }

    #[test]
    fn an_installer_item_past_the_list_is_refused() {
        let install = crate::test_support::own::install();
        let _lock = crate::test_support::test_lock();
        let past = std::panic::catch_unwind(AssertUnwindSafe(|| {
            ListMenu::installer(install, ListId::Room).item(3)
        }));
        assert!(past.is_err());
    }
}
