//! A list menu (`install::run_page`'s pages, the replay viewer's items) and its items.
//!
//! A list is a header of 0x18 bytes (+0 flags, +1 the item count, +2 the title's string,
//! +4 the item highlighted, +8 the list's action, +0x10 the neighbours table for
//! `menu::list_step`, +0x14 a key action) and its items of 0x10 bytes from +0x18 (+0 kind in
//! the low nibble and marks in the high, +1 the next list or the number of choices, +2 the
//! text, +4 the item's box, +8 the byte or text it edits or the link's action, +0xC the
//! choices, the edit record or the least value).
//!
//! The installer's lists are typed records (`ListData`, `Item`, `EditRecord`) in
//! `install::vars::InstallerPages`, one per `ListId`; their words that name data (boxes,
//! texts, edit records) are links (`install::page_links::PageLink`) each accessor resolves by
//! kind, and the words that name code stay as read. The replay viewer's list is a `ListData26`
//! in the replay-view block. An item the list does not hold is refused.

use crate::install::installer::refuse;
use crate::install::installer::{rb, rw, wb};
use crate::install::page_links::{ListId, PageLink};
use crate::place::BytePlace;
use crate::session::{PlainByte, PlainDword, PlainWord};

/// The header's bytes.
const HEADER: u32 = 0x18;
/// An item's bytes.
const ITEM: u32 = 0x10;

/// A list's header as the installer's pages hold it (see the module: 0x18 bytes).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView, rc2k_derive::ZeroCells)]
pub struct ListHeader {
    /// +0: the flags.
    pub flags: PlainByte,
    /// +1: the number of items.
    pub count: PlainByte,
    /// +2: the title's string.
    pub title_text: PlainWord,
    /// +4: the item highlighted.
    pub highlight: PlainDword,
    /// +8: the list's action.
    pub action: PlainDword,
    /// +0xC: nothing reads it.
    pub unused_0c: PlainDword,
    /// +0x10: the link to the neighbours table (0 for none).
    pub neighbours: PlainDword,
    /// +0x14: the key action.
    pub key_action: PlainDword,
}

impl ListHeader {
    // One argument a field, in the order of the header's bytes.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        flags: u8,
        count: u8,
        title_text: u16,
        highlight: u32,
        action: u32,
        unused_0c: u32,
        neighbours: u32,
        key_action: u32,
    ) -> Self {
        Self {
            flags: PlainByte::new(flags),
            count: PlainByte::new(count),
            title_text: PlainWord::new(title_text),
            highlight: PlainDword::new(highlight),
            action: PlainDword::new(action),
            unused_0c: PlainDword::new(unused_0c),
            neighbours: PlainDword::new(neighbours),
            key_action: PlainDword::new(key_action),
        }
    }
}

/// A list's item as the installer's pages hold it (see the module: 0x10 bytes).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView, rc2k_derive::ZeroCells)]
pub struct Item {
    /// +0: the kind (low nibble) and marks (high bits).
    pub first: PlainByte,
    /// +1: the next list or the number of choices.
    pub count: PlainByte,
    /// +2: the item's text (a string id).
    pub text: PlainWord,
    /// +4: the link to the item's box.
    pub box_link: PlainDword,
    /// +8: the link or action, or where the item's value or text is.
    pub target: PlainDword,
    /// +0xC: the choices, the edit record or the text action.
    pub data: PlainDword,
}

impl Item {
    pub const fn new(
        first: u8,
        count: u8,
        text: u16,
        box_link: u32,
        target: u32,
        data: u32,
    ) -> Self {
        Self {
            first: PlainByte::new(first),
            count: PlainByte::new(count),
            text: PlainWord::new(text),
            box_link: PlainDword::new(box_link),
            target: PlainDword::new(target),
            data: PlainDword::new(data),
        }
    }
}

/// Defines a list of a fixed number of items: its header and the items after it (the derive
/// takes no const generics).
macro_rules! list_data {
    ($($name:ident: $n:expr),* $(,)?) => {$(
        #[repr(C)]
        #[derive(rc2k_derive::Cells)]
        #[derive(rc2k_derive::ByteView)]
        #[derive(rc2k_derive::ZeroCells)]
        pub struct $name {
            pub header: ListHeader,
            pub items: [Item; $n],
        }

        impl $name {
            pub const fn new(header: ListHeader, items: [Item; $n]) -> Self {
                Self { header, items }
            }
        }
    )*};
}

list_data! {
    ListData3: 3,
    ListData4: 4,
    ListData5: 5,
    ListData6: 6,
    ListData10: 10,
    ListData18: 18,
    ListData26: 26,
}

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

/// A list menu: a header and the items it holds (one of the installer's lists, or the replay
/// viewer's).
#[derive(Clone, Copy)]
pub struct ListMenu {
    header: &'static ListHeader,
    items: &'static [Item],
}

impl core::fmt::Debug for ListMenu {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match ListId::ALL
            .into_iter()
            .find(|&list| core::ptr::eq(list_header(list), self.header))
        {
            Some(list) => write!(f, "ListMenu({list:?})"),
            None => write!(f, "ListMenu(the viewer's)"),
        }
    }
}

impl PartialEq for ListMenu {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.header, other.header)
    }
}

impl Eq for ListMenu {}

/// The header and items of one of the installer's lists. The pages that are only an action
/// have no items.
fn list_parts(list: ListId) -> (&'static ListHeader, &'static [Item]) {
    let pages = &crate::install::page_links::installer().pages;
    match list {
        ListId::Language => (&pages.language_menu.header, &pages.language_menu.items),
        ListId::Main => (&pages.main_menu.header, &pages.main_menu.items),
        ListId::Mode => (&pages.mode_menu.header, &pages.mode_menu.items),
        ListId::Drives8 => (&pages.drive_menu_8.header, &pages.drive_menu_8.items),
        ListId::Drives16 => (&pages.drive_menu_16.header, &pages.drive_menu_16.items),
        ListId::DrivesMore => (&pages.menu_more.header, &pages.menu_more.items),
        ListId::Folder => (&pages.folder_menu.header, &pages.folder_menu.items),
        ListId::Room => (&pages.room_menu.header, &pages.room_menu.items),
        ListId::Dxsetup => (&pages.dxsetup_menu.header, &pages.dxsetup_menu.items),
        ListId::Finish => (&pages.finish_menu.header, &pages.finish_menu.items),
        ListId::InstallFiles => (&pages.action_pages[0], &[]),
        ListId::Uninstall => (&pages.action_pages[1], &[]),
        ListId::OpenInstalled => (&pages.action_pages[2], &[]),
        ListId::AskRegistration => (&pages.action_pages[3], &[]),
    }
}

/// The header of one of the installer's lists.
pub(crate) fn list_header(list: ListId) -> &'static ListHeader {
    list_parts(list).0
}

/// The typed items of one of the installer's lists (none for the action pages).
pub(crate) fn list_items(list: ListId) -> &'static [Item] {
    list_parts(list).1
}

impl ListMenu {
    /// The list a link of the installer's pages names; a link to anything else is refused (the
    /// page walk takes lists only from the first word of a page entry).
    pub fn at(link: PageLink) -> Result<Self, crate::errors::BadInput> {
        match link.list_id() {
            Some(list) => Ok(Self::installer(list)),
            None => Err(crate::errors::BadInput(
                "a page link that is no list of the installer's pages",
            )),
        }
    }

    /// One of the installer's lists.
    pub fn installer(list: ListId) -> Self {
        let (header, items) = list_parts(list);
        Self { header, items }
    }

    /// The replay viewer's list.
    pub fn viewer(list: &'static ListData26) -> Self {
        Self {
            header: &list.header,
            items: &list.items,
        }
    }

    /// +0: the flags (bit 4 the page stays on Escape, bits 6 and 7 the page hidden).
    pub fn flags(self) -> u8 {
        self.header.flags.get()
    }

    pub fn set_flags(self, flags: u8) {
        self.header.flags.set(flags);
    }

    /// +1: the number of items.
    pub fn count(self) -> u8 {
        self.header.count.get()
    }

    pub fn set_count(self, count: u8) {
        self.header.count.set(count);
    }

    /// +2: the title's string.
    pub fn title_text(self) -> u16 {
        self.header.title_text.get()
    }

    /// +4: the item highlighted.
    pub fn highlight(self) -> u32 {
        self.header.highlight.get()
    }

    pub fn set_highlight(self, highlight: u32) {
        self.header.highlight.set(highlight);
    }

    /// +8: the list's action (`install::installer::page_action`).
    pub fn action(self) -> u32 {
        self.header.action.get()
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

    /// +0x14: the key action (`install::installer::key_action`).
    pub fn key_action(self) -> u32 {
        self.header.key_action.get()
    }

    /// Item `n`. A list holds the items its record has: an item past them is refused (the
    /// lists' counts stay within them, `the_counts_stay_within_the_items`).
    pub fn item(self, n: u32) -> ListItem {
        match self.items.get(n as usize) {
            Some(item) => ListItem(item),
            None => refuse(crate::errors::BadInput("an item past the items of a list")),
        }
    }

    /// The item whose first byte is `offset` from the list (the pages' tables of marks
    /// give offsets, 0x28 the second item). An offset that is no item's start is refused.
    pub fn item_at_offset(self, offset: u32) -> ListItem {
        let past = offset.wrapping_sub(HEADER);
        if !past.is_multiple_of(ITEM) {
            refuse(crate::errors::BadInput(
                "an offset in a list that is no item's start",
            ));
        }
        self.item(past / ITEM)
    }
}

/// One item of a list menu.
#[derive(Clone, Copy)]
pub struct ListItem(&'static Item);

impl core::fmt::Debug for ListItem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match Self::position(self.0) {
            Some(ItemPosition::Title) => write!(f, "ListItem(title)"),
            Some(ItemPosition::In(list, n)) => write!(f, "ListItem({list:?} #{n})"),
            None => write!(f, "ListItem(the viewer's)"),
        }
    }
}

impl PartialEq for ListItem {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.0, other.0)
    }
}

impl Eq for ListItem {}

/// Which typed item of the installer's pages an item is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemPosition {
    /// The title's item.
    Title,
    /// Item `.1` of list `.0`.
    In(ListId, u32),
}

/// The kind of the handle a piece keeps for its item (the high byte; 0 is no item).
const ITEM_HANDLE: u32 = 7;
/// The list number the title's item is kept under (no list has it).
const TITLE_LIST: u32 = 0xFF;

impl ListItem {
    /// Where `item`, one of the pages' typed items, is.
    fn position(item: &Item) -> Option<ItemPosition> {
        if core::ptr::eq(
            item,
            &raw const crate::install::page_links::installer().pages.title,
        ) {
            return Some(ItemPosition::Title);
        }
        ListId::ALL.into_iter().find_map(|list| {
            list_items(list)
                .iter()
                .position(|typed| core::ptr::eq(typed, item))
                .map(|n| ItemPosition::In(list, n as u32))
        })
    }

    /// Which typed item of the installer's pages this is (none for the viewer's items).
    pub fn installer_position(self) -> Option<ItemPosition> {
        Self::position(self.0)
    }

    /// The item a handle names (`to_handle`): what a piece keeps of its item. A word that
    /// names no item of the installer's pages is refused.
    pub fn from_handle(handle: u32) -> Result<Self, crate::errors::BadInput> {
        let refused = || crate::errors::BadInput("an item that is none of the installer's items");
        if handle >> 24 != ITEM_HANDLE {
            return Err(refused());
        }
        let (list, n) = ((handle >> 8) & 0xFFFF, handle & 0xFF);
        if list == TITLE_LIST {
            return if n == 0 {
                Ok(Self::of_title())
            } else {
                Err(refused())
            };
        }
        let list = ListId::ALL.get(list as usize).ok_or_else(refused)?;
        let item = list_items(*list).get(n as usize).ok_or_else(refused)?;
        Ok(Self(item))
    }

    /// The handle a piece keeps for this item; an item that is none of the installer's is
    /// refused.
    pub fn to_handle(self) -> Result<u32, crate::errors::BadInput> {
        match self.installer_position() {
            Some(ItemPosition::Title) => Ok(ITEM_HANDLE << 24 | TITLE_LIST << 8),
            Some(ItemPosition::In(list, n)) => Ok(ITEM_HANDLE << 24 | (list as u32) << 8 | n),
            None => Err(crate::errors::BadInput(
                "an item that is none of the installer's items",
            )),
        }
    }

    /// The title's item (the pages' `title` record).
    pub fn of_title() -> Self {
        Self(&crate::install::page_links::installer().pages.title)
    }

    /// +0: the kind (low nibble) and marks (high bits), whole.
    pub fn first(self) -> u8 {
        self.0.first.get()
    }

    pub fn set_first(self, first: u8) {
        self.0.first.set(first);
    }

    /// The kind: the low nibble of +0.
    pub fn kind(self) -> u8 {
        self.first() & 0xF
    }

    /// +1: the next list (a link) or the number of choices.
    pub fn count(self) -> u8 {
        self.0.count.get()
    }

    /// +2: the item's text (a string id).
    pub fn text_id(self) -> u16 {
        self.0.text.get()
    }

    /// +1 of the replay viewer's items: the viewer action Enter does there.
    pub fn viewer_action(self) -> u8 {
        self.count()
    }

    /// +1 of a text item: the kind of its font.
    pub fn font_kind(self) -> u8 {
        self.count()
    }

    /// +4: the box the item's pieces are placed in (none: the title's).
    pub fn box_link(self) -> PageLink {
        PageLink::from_raw(self.0.box_link.get())
    }

    /// +8: the link's or action's address, or where the item's value or text is.
    pub fn target(self) -> u32 {
        self.0.target.get()
    }

    /// +8 as the link.
    fn target_link(self) -> PageLink {
        PageLink::from_raw(self.target())
    }

    /// +8 as the text the item edits or shows (a link).
    pub fn text(self) -> BytePlace {
        self.link_place()
    }

    /// +8 as the byte the item sets (a choice, a value; a link).
    pub fn value(self) -> ItemByte {
        ItemByte(self.link_place())
    }

    /// The place the link at +8 names: the folder typed.
    fn link_place(self) -> BytePlace {
        self.target_link().place().unwrap_or_else(|e| refuse(e))
    }

    /// +0xC: the pointer to the choices, the edit record or the text action.
    pub fn data(self) -> u32 {
        self.0.data.get()
    }

    /// +0xC as the byte the least value is (a value item).
    pub fn value_least(self) -> u8 {
        self.0.data.get().to_le_bytes()[0]
    }

    /// +0xD: the most value (a value item).
    pub fn value_most(self) -> u8 {
        self.0.data.get().to_le_bytes()[1]
    }

    /// +0xC as the item's choices (a choice item; a link): the table is the place the link
    /// names (`PageLink::place`), read on past the table (see `Choices`).
    pub fn choices(self) -> Choices {
        Choices(
            PageLink::from_raw(self.data())
                .place()
                .unwrap_or_else(|e| refuse(e)),
        )
    }

    /// +0xC as the item's edit record (an edit item).
    pub fn edit_field(self) -> EditField {
        EditField(
            PageLink::from_raw(self.data())
                .edit_record()
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

    pub fn set_flags(self, flags: u8) {
        self.0.flags.set(flags);
    }

    pub fn most(self) -> u8 {
        self.0.most.get()
    }

    pub fn default_text(self) -> u16 {
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
            ListId::Main | ListId::Mode => 5,
            ListId::Drives16 => 18,
            ListId::DrivesMore => 26,
            ListId::Folder => 6,
            ListId::Room => 3,
            ListId::Dxsetup | ListId::Finish => 4,
            _ => 0,
        }
    }

    #[test]
    fn a_pieces_item_is_kept_as_a_handle() {
        let _lock = crate::test_support::test_lock();
        let title = ListItem::of_title();
        assert_eq!(
            ListItem::from_handle(title.to_handle().unwrap()).unwrap(),
            title
        );
        for list in ListId::ALL {
            for n in 0..held(list) {
                let item = ListMenu::installer(list).item(n);
                assert_eq!(item.installer_position(), Some(ItemPosition::In(list, n)));
                assert_eq!(
                    ListItem::from_handle(item.to_handle().unwrap()).unwrap(),
                    item
                );
            }
        }
    }

    #[test]
    fn a_handle_that_names_no_item_is_refused() {
        let _lock = crate::test_support::test_lock();
        assert!(ListItem::from_handle(0).is_err());
        assert!(ListItem::from_handle(u32::MAX).is_err());
        assert!(ListItem::from_handle(ITEM_HANDLE << 24 | 14 << 8).is_err());
        assert!(ListItem::from_handle(ITEM_HANDLE << 24 | TITLE_LIST << 8 | 1).is_err());
        let past = ITEM_HANDLE << 24 | (ListId::Room as u32) << 8 | held(ListId::Room);
        assert!(ListItem::from_handle(past).is_err());
    }

    /// M4 for the installer's lists: every list's count starts within the items its record
    /// holds, so no page asks for an item past them, which is refused. The drive lists' counts
    /// are set by `list_drives` (the drives and two more, at most 24 drives: `drives::kept`).
    #[test]
    fn the_counts_stay_within_the_items() {
        let _lock = crate::test_support::test_lock();
        for list in ListId::ALL {
            let menu = ListMenu::installer(list);
            assert!(u32::from(menu.count()) <= held(list), "{list:?}");
            assert_eq!(list_items(list).len() as u32, held(list), "{list:?}");
        }
    }

    /// M4 for the choice items: no item of the installer's lists is a choice item (kind 2),
    /// and the only run-time write of an item's kind is the main page's first item, made
    /// kind 5 (`check_at_start::set_script_entry`). The viewer's items are kind 1 or 5 in
    /// their starting data and only their marks (the high bits) change. The pages hold no
    /// table a choice link could name either: only the typed folder's link is a place.
    #[test]
    fn the_lists_hold_no_choice_items() {
        let _lock = crate::test_support::test_lock();
        for list in ListId::ALL {
            let menu = ListMenu::installer(list);
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
        let _lock = crate::test_support::test_lock();
        let room = &crate::install::page_links::installer()
            .pages
            .room_menu
            .items[0];
        let saved = (room.first.get(), room.data.get());
        room.first.set(2);
        for link in [
            PageLink::table(0),
            PageLink::list(ListId::Main),
            PageLink::box_at(8),
            PageLink::folder_edit(),
            PageLink::from_raw(0),
        ] {
            room.data.set(link.raw());
            let item = ListMenu::installer(ListId::Room).item(0);
            assert!(
                std::panic::catch_unwind(AssertUnwindSafe(|| item.choices())).is_err(),
                "{link:?}"
            );
        }
        room.data.set(PageLink::typed(2).raw());
        let item = ListMenu::installer(ListId::Room).item(0);
        assert!(std::panic::catch_unwind(AssertUnwindSafe(|| item.choices().marks(0))).is_ok());
        room.first.set(saved.0);
        room.data.set(saved.1);
    }

    #[test]
    fn an_installer_item_past_the_list_is_refused() {
        let _lock = crate::test_support::test_lock();
        let past = std::panic::catch_unwind(AssertUnwindSafe(|| {
            ListMenu::installer(ListId::Room).item(3)
        }));
        assert!(past.is_err());
    }
}
