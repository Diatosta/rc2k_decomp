//! The record layout of a list menu and its items, as the installer's pages and the replay
//! viewer's list hold them.
//!
//! A list is a header of 0x18 bytes (+0 flags, +1 the item count, +2 the title's string,
//! +4 the item highlighted, +8 the list's action, +0x10 the neighbours table for
//! `menu::list_step`, +0x14 a key action) and its items of 0x10 bytes from +0x18 (+0 kind in
//! the low nibble and marks in the high, +1 the next list or the number of choices, +2 the
//! text, +4 the item's box, +8 the byte or text it edits or the link's action, +0xC the
//! choices, the edit record or the least value).
//!
//! The words of an item that name data are links the installer's pages resolve
//! (`install::list_pages`); the replay viewer's list is a `ListData26` in the replay-view
//! block. An item the list does not hold is refused.

use crate::memory::session::{PlainByte, PlainDword, PlainWord};

/// The header's bytes.
const HEADER: u32 = 0x18;
/// An item's bytes.
const ITEM: u32 = 0x10;

/// A list's header as the installer's pages hold it (see the module: 0x18 bytes).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView, rc2k_derive::ZeroCells)]
pub struct ListHeader {
    /// The flags.
    pub flags: PlainByte,
    /// The number of items.
    pub count: PlainByte,
    /// The title's string.
    pub title_text: PlainWord,
    /// The item highlighted.
    pub highlight: PlainDword,
    /// Nothing reads it (what a page does is `install::run_page`'s to know).
    pub action: PlainDword,
    /// Nothing reads it.
    pub unused_0c: PlainDword,
    /// The link to the neighbours table (0 for none).
    pub neighbours: PlainDword,
    /// Nothing reads it.
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
    /// The kind (low nibble) and marks (high bits).
    pub first: PlainByte,
    /// The next list or the number of choices.
    pub count: PlainByte,
    /// The item's text (a string id).
    pub text: PlainWord,
    /// The link to the item's box.
    pub box_link: PlainDword,
    /// The link or action, or where the item's value or text is.
    pub target: PlainDword,
    /// The choices, the edit record or the text action.
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
    ListData6: 6,
    ListData10: 10,
    ListData18: 18,
    ListData26: 26,
}

/// A list menu: a header and the items it holds (one of the installer's lists, or the replay
/// viewer's).
#[derive(Clone, Copy)]
pub struct ListMenu {
    pub(crate) header: &'static ListHeader,
    pub(crate) items: &'static [Item],
}

impl core::fmt::Debug for ListMenu {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ListMenu({:p})", self.header)
    }
}

impl PartialEq for ListMenu {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.header, other.header)
    }
}

impl Eq for ListMenu {}

impl ListMenu {
    /// The replay viewer's list.
    pub fn viewer(list: &'static ListData26) -> Self {
        Self {
            header: &list.header,
            items: &list.items,
        }
    }

    /// The flags (bit 4 the page stays on Escape, bits 6 and 7 the page hidden).
    pub fn flags(self) -> u8 {
        self.header.flags.get()
    }

    /// The number of items.
    pub fn count(self) -> u8 {
        self.header.count.get()
    }

    pub fn set_count(self, count: u8) {
        self.header.count.set(count);
    }

    /// The title's string.
    pub fn title_text(self) -> u16 {
        self.header.title_text.get()
    }

    /// The item highlighted.
    pub fn highlight(self) -> u32 {
        self.header.highlight.get()
    }

    pub fn set_highlight(self, highlight: u32) {
        self.header.highlight.set(highlight);
    }

    /// Item `n`. A list holds the items its record has: an item past them is refused (the
    /// lists' counts stay within them, `the_counts_stay_within_the_items`).
    pub fn item(self, n: u32) -> ListItem {
        match self.items.get(n as usize) {
            Some(item) => ListItem(item),
            None => crate::memory::host_hooks::end(
                "list menu",
                crate::errors::BadInput("an item past the items of a list"),
            ),
        }
    }

    /// The item whose first byte is `offset` from the list (the pages' tables of marks
    /// give offsets, 0x28 the second item). An offset that is no item's start is refused.
    pub fn item_at_offset(self, offset: u32) -> ListItem {
        let past = offset.wrapping_sub(HEADER);
        if !past.is_multiple_of(ITEM) {
            crate::memory::host_hooks::end(
                "list menu",
                crate::errors::BadInput("an offset in a list that is no item's start"),
            );
        }
        self.item(past / ITEM)
    }
}

/// One item of a list menu.
#[derive(Clone, Copy)]
pub struct ListItem(pub(crate) &'static Item);

impl core::fmt::Debug for ListItem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ListItem({:p})", self.0)
    }
}

impl PartialEq for ListItem {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.0, other.0)
    }
}

impl Eq for ListItem {}

impl ListItem {
    /// The kind (low nibble) and marks (high bits), whole.
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

    /// The next list (a link) or the number of choices.
    pub fn count(self) -> u8 {
        self.0.count.get()
    }

    /// The item's text (a string id).
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

    /// The link's or action's address, or where the item's value or text is.
    pub fn target(self) -> u32 {
        self.0.target.get()
    }

    /// The pointer to the choices, the edit record or the text action.
    pub fn data(self) -> u32 {
        self.0.data.get()
    }

    /// +0xC as the byte the least value is (a value item).
    pub fn value_least(self) -> u8 {
        self.0.data.get().to_le_bytes()[0]
    }

    /// The most value (a value item).
    pub fn value_most(self) -> u8 {
        self.0.data.get().to_le_bytes()[1]
    }
}
