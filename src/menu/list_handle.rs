//! A list menu (`install::run_page`'s pages, the replay viewer's items) and its items by name, in
//! place of `list + 0x18 + 0x10 * n` sums on game addresses.
//!
//! A list is a header of 0x18 bytes (+0 flags, +1 the item count, +2 the title's string,
//! +4 the item highlighted, +8 the list's action, +0x10 the neighbours table for
//! `menu::list_step`, +0x14 a key action) and its items of 0x10 bytes from +0x18 (+0 kind in
//! the low nibble and marks in the high, +1 the next list or the number of choices, +2 the
//! text, +4 the item's box, +8 the byte or text it edits or the link's action, +0xC the
//! choices, the edit record or the least value). Every accessor reads or writes inside its
//! own 0x18 header bytes or 0x10 item bytes; a place outside the bytes it is read from (the
//! installer's pages, or the record holding the replay viewer's list) ends the program
//! (`install::installer::refuse`). The words that name data
//! (boxes, tables, texts, values) are links (`install::page_links::PageLink`), handles the
//! installer's pages hold in place of addresses and each accessor resolves with bounds; the
//! words that name code stay as read.
//!
//! The item count is the list's own and is not checked against the item asked for: the
//! original goes past it too (a list of no items goes round 2^32 times). An item past the
//! list's items is the bytes that follow the list in the installer's pages (the next list,
//! table or record), read as an item: `item` and `Choices` reach them through a place bounded
//! by the whole of the pages, not by the list.
//!
//! The installer's own lists are typed records (`ListData`, `Item`, `EditRecord`) in
//! `install::vars::InstallerPages`, with the layout of these bytes.

use crate::install::installer::refuse;
use crate::install::installer::{rb, rd, rl, rw, wb, wd};
use crate::install::page_links::{ListId, PageLink};
use crate::place::BytePlace;
use crate::session::{PlainByte, PlainDword, PlainWord};

/// The header's bytes.
const HEADER: u32 = 0x18;
/// An item's bytes.
const ITEM: u32 = 0x10;

/// A list's header as the installer's pages hold it (see the module: 0x18 bytes).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ListHeader {
    /// +0: the flags.
    pub flags: PlainByte,
    /// +1: the number of items.
    pub count: PlainByte,
    /// +2: the title's string.
    pub title_text: PlainWord,
    /// +4: the item highlighted.
    pub highlight: PlainDword,
    /// +8: the list's action (an address in RAL.EXE's code).
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
#[derive(rc2k_derive::Cells)]
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
#[derive(rc2k_derive::Cells)]
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
#[derive(rc2k_derive::Cells)]
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

/// Where a list menu's header is: one of the installer's lists, typed with its items, or
/// bytes read as a header.
#[derive(Clone, Copy)]
enum Head {
    /// A list of the installer's pages: its header and the items it holds.
    Pages(&'static ListHeader, &'static [Item]),
    /// The 0x18 header bytes at a place: the replay viewer's list, or a thing of the
    /// installer's pages that is no list (the page walk looks at what a link names).
    Bytes(Raw),
}

/// Bytes read as a header or an item: at an offset of the installer's pages, or at a place.
#[derive(Clone, Copy)]
enum Raw {
    /// The byte offset in the installer's pages (wrapping as the original's address sum).
    Pages(u32),
    Place(BytePlace),
}

impl Raw {
    fn place(self) -> BytePlace {
        match self {
            Raw::Pages(offset) => BytePlace::in_store(
                &crate::install::page_links::installer().pages,
                offset as usize,
            ),
            Raw::Place(at) => at,
        }
    }

    /// `n` bytes on.
    fn add(self, n: u32) -> Raw {
        match self {
            Raw::Pages(offset) => Raw::Pages(offset.wrapping_add(n)),
            Raw::Place(at) => Raw::Place(at.add(n)),
        }
    }

    /// The `len` bytes at `offset` from here, inside a header or an item.
    fn field(self, offset: u32, len: u32) -> BytePlace {
        debug_assert!(offset + len <= HEADER);
        self.add(offset).place()
    }
}

impl core::fmt::Debug for Raw {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Raw::Pages(offset) => write!(f, "pages +{offset:#x}"),
            Raw::Place(at) => write!(f, "{at:?}"),
        }
    }
}

impl PartialEq for Raw {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Raw::Pages(a), Raw::Pages(b)) => a == b,
            (Raw::Place(a), Raw::Place(b)) => a.same_place(*b),
            _ => false,
        }
    }
}

/// A list menu.
#[derive(Clone, Copy)]
pub struct ListMenu(Head);

impl core::fmt::Debug for ListMenu {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            Head::Pages(header, _) => write!(f, "ListMenu(pages +{:#x})", pages_offset_of(header)),
            Head::Bytes(at) => write!(f, "ListMenu({at:?})"),
        }
    }
}

impl PartialEq for ListMenu {
    fn eq(&self, other: &Self) -> bool {
        match (self.0, other.0) {
            (Head::Pages(a, _), Head::Pages(b, _)) => core::ptr::eq(a, b),
            (Head::Bytes(a), Head::Bytes(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for ListMenu {}

/// Where `thing`, a field of the installer's pages, starts in them.
fn pages_offset_of<T>(thing: &T) -> usize {
    let pages = &crate::install::page_links::installer().pages;
    (std::ptr::from_ref(thing) as usize).wrapping_sub(std::ptr::from_ref(pages) as usize)
}

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

impl ListMenu {
    /// The list a link of the installer's pages names. Any thing of the pages is taken: the
    /// page walk reads the words after an entry's answers as links and looks at the list they
    /// name, as the original does (`install::run_pages::pass_over`); what is no list is read
    /// as the bytes of a header.
    pub fn at(link: PageLink) -> Result<Self, crate::errors::BadInput> {
        match link.list_id() {
            Some(list) => Ok(Self::installer(list)),
            None => Ok(Self(Head::Bytes(Raw::Pages(link.pages_start()? as u32)))),
        }
    }

    /// One of the installer's lists.
    pub fn installer(list: ListId) -> Self {
        let (header, items) = list_parts(list);
        Self(Head::Pages(header, items))
    }

    /// The list at a place in the game's data (the replay viewer's, which is a run of dwords
    /// and not typed as a list).
    pub fn of(place: impl Into<BytePlace>) -> Self {
        Self(Head::Bytes(Raw::Place(place.into())))
    }

    /// A header field's bytes: `len` bytes at `offset`, inside the header.
    fn head(at: Raw, offset: u32, len: u32) -> BytePlace {
        at.field(offset, len)
    }

    /// +0: the flags (bit 4 the page stays on Escape, bits 6 and 7 the page hidden).
    pub fn flags(self) -> u8 {
        match self.0 {
            Head::Pages(header, _) => header.flags.get(),
            Head::Bytes(at) => rb(Self::head(at, 0, 1)),
        }
    }

    pub fn set_flags(self, flags: u8) {
        match self.0 {
            Head::Pages(header, _) => header.flags.set(flags),
            Head::Bytes(at) => wb(Self::head(at, 0, 1), flags),
        }
    }

    /// +1: the number of items.
    pub fn count(self) -> u8 {
        match self.0 {
            Head::Pages(header, _) => header.count.get(),
            Head::Bytes(at) => rb(Self::head(at, 1, 1)),
        }
    }

    pub fn set_count(self, count: u8) {
        match self.0 {
            Head::Pages(header, _) => header.count.set(count),
            Head::Bytes(at) => wb(Self::head(at, 1, 1), count),
        }
    }

    /// +2: the title's string.
    pub fn title_text(self) -> u16 {
        match self.0 {
            Head::Pages(header, _) => header.title_text.get(),
            Head::Bytes(at) => rw(Self::head(at, 2, 2)),
        }
    }

    /// +4: the item highlighted.
    pub fn highlight(self) -> u32 {
        match self.0 {
            Head::Pages(header, _) => header.highlight.get(),
            Head::Bytes(at) => rd(Self::head(at, 4, 4)),
        }
    }

    pub fn set_highlight(self, highlight: u32) {
        match self.0 {
            Head::Pages(header, _) => header.highlight.set(highlight),
            Head::Bytes(at) => wd(Self::head(at, 4, 4), highlight),
        }
    }

    /// +8: the list's action (an address in RAL.EXE's code: `install::installer::page_action`).
    pub fn action(self) -> u32 {
        match self.0 {
            Head::Pages(header, _) => header.action.get(),
            Head::Bytes(at) => rd(Self::head(at, 8, 4)),
        }
    }

    /// +0x10: the neighbours table for `menu::list_step` (None for none).
    pub fn neighbours(self) -> Option<BytePlace> {
        let link = match self.0 {
            Head::Pages(header, _) => PageLink::from_raw(header.neighbours.get()),
            Head::Bytes(at) => rl(Self::head(at, 0x10, 4)),
        };
        (!link.is_none()).then(|| link.in_pages().unwrap_or_else(|e| refuse(e)))
    }

    /// +0x14: the key action (`install::installer::key_action`).
    pub fn key_action(self) -> u32 {
        match self.0 {
            Head::Pages(header, _) => header.key_action.get(),
            Head::Bytes(at) => rd(Self::head(at, 0x14, 4)),
        }
    }

    /// Item `n`, from +0x18, 0x10 bytes each (wrapping as the original's sum). The item count
    /// is not checked: an item past the list's items is the bytes that follow the list in the
    /// installer's pages (the next list, table or record), read as an item.
    pub fn item(self, n: u32) -> ListItem {
        let start = match self.0 {
            Head::Pages(_, items) if (n as usize) < items.len() => {
                return ListItem(Spot::Typed(&items[n as usize]));
            }
            Head::Pages(header, _) => Raw::Pages(pages_offset_of(header) as u32),
            Head::Bytes(at) => at,
        };
        ListItem(Spot::Bytes(
            start.add(HEADER.wrapping_add(ITEM.wrapping_mul(n))),
        ))
    }

    /// The item whose first byte is `offset` from the list (the pages' tables of marks
    /// give offsets, 0x28 the second item). An offset that is no item's start is refused.
    pub fn item_at_offset(self, offset: u32) -> ListItem {
        let past = offset.wrapping_sub(HEADER);
        if !past.is_multiple_of(ITEM) {
            crate::install::installer::refuse(crate::errors::BadInput(
                "an offset in a list that is no item's start",
            ));
        }
        self.item(past / ITEM)
    }
}

/// Where an item is: typed in one of the installer's lists, or its 0x10 bytes at a place.
#[derive(Clone, Copy)]
enum Spot {
    Typed(&'static Item),
    /// An item read past a list's items, or at an offset of the installer's pages that holds
    /// no typed item.
    Bytes(Raw),
}

/// One item of a list menu (its 0x10 bytes).
#[derive(Clone, Copy)]
pub struct ListItem(Spot);

impl core::fmt::Debug for ListItem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            Spot::Typed(item) => write!(f, "ListItem(pages +{:#x})", pages_offset_of(item)),
            Spot::Bytes(at) => write!(f, "ListItem({at:?})"),
        }
    }
}

impl PartialEq for ListItem {
    fn eq(&self, other: &Self) -> bool {
        match (self.0, other.0) {
            (Spot::Typed(a), Spot::Typed(b)) => core::ptr::eq(a, b),
            (Spot::Bytes(a), Spot::Bytes(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for ListItem {}

impl ListItem {
    /// The item `offset` bytes into the installer's pages (`crate::install::vars::
    /// InstallerPages`), where every list of the installer lives: what a piece keeps of its
    /// item. An offset with no item's room before the pages' end is refused.
    pub fn in_pages(offset: u32) -> Result<Self, crate::errors::BadInput> {
        use crate::install::vars::InstallerPages;
        if (offset as usize).saturating_add(ITEM as usize) > size_of::<InstallerPages>() {
            return Err(crate::errors::BadInput(
                "an item outside the installer's pages",
            ));
        }
        if offset as usize == core::mem::offset_of!(InstallerPages, title) {
            return Ok(Self::of_title());
        }
        for (list, list_at) in ListId::ALL.map(|(list, _)| (list, list.offset())) {
            let (_, items) = list_parts(list);
            let first = list_at + HEADER as usize;
            let past = (offset as usize).wrapping_sub(first);
            if past < items.len() * ITEM as usize && past.is_multiple_of(ITEM as usize) {
                return Ok(Self(Spot::Typed(&items[past / ITEM as usize])));
            }
        }
        // The bytes of the pages that are no typed item (a table, a record, a list's header).
        Ok(Self(Spot::Bytes(Raw::Pages(offset))))
    }

    /// The title's item (the pages' `title` record).
    pub fn of_title() -> Self {
        Self(Spot::Typed(
            &crate::install::page_links::installer().pages.title,
        ))
    }

    /// Where this item is in the installer's pages (see `in_pages`); an item outside them is
    /// refused.
    pub fn pages_offset(self) -> Result<u32, crate::errors::BadInput> {
        use crate::install::vars::InstallerPages;
        let offset = match self.0 {
            Spot::Typed(item) => pages_offset_of(item) as u32,
            Spot::Bytes(Raw::Pages(offset)) => offset,
            Spot::Bytes(Raw::Place(_)) => {
                return Err(crate::errors::BadInput(
                    "an item outside the installer's pages",
                ));
            }
        };
        if (offset as usize).saturating_add(ITEM as usize) > size_of::<InstallerPages>() {
            return Err(crate::errors::BadInput(
                "an item outside the installer's pages",
            ));
        }
        Ok(offset)
    }

    fn field(at: Raw, offset: u32, len: u32) -> BytePlace {
        debug_assert!(offset + len <= ITEM);
        at.field(offset, len)
    }

    /// +0: the kind (low nibble) and marks (high bits), whole.
    pub fn first(self) -> u8 {
        match self.0 {
            Spot::Typed(item) => item.first.get(),
            Spot::Bytes(at) => rb(Self::field(at, 0, 1)),
        }
    }

    pub fn set_first(self, first: u8) {
        match self.0 {
            Spot::Typed(item) => item.first.set(first),
            Spot::Bytes(at) => wb(Self::field(at, 0, 1), first),
        }
    }

    /// The kind: the low nibble of +0.
    pub fn kind(self) -> u8 {
        self.first() & 0xF
    }

    /// +1: the next list (a link) or the number of choices.
    pub fn count(self) -> u8 {
        match self.0 {
            Spot::Typed(item) => item.count.get(),
            Spot::Bytes(at) => rb(Self::field(at, 1, 1)),
        }
    }

    /// +2: the item's text (a string id).
    pub fn text_id(self) -> u16 {
        match self.0 {
            Spot::Typed(item) => item.text.get(),
            Spot::Bytes(at) => rw(Self::field(at, 2, 2)),
        }
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
        match self.0 {
            Spot::Typed(item) => PageLink::from_raw(item.box_link.get()),
            Spot::Bytes(at) => rl(Self::field(at, 4, 4)),
        }
    }

    /// +8: the link's or action's address, or where the item's value or text is.
    pub fn target(self) -> u32 {
        match self.0 {
            Spot::Typed(item) => item.target.get(),
            Spot::Bytes(at) => rd(Self::field(at, 8, 4)),
        }
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

    /// The place the link at +8 names, in the pages or the folder typed.
    fn link_place(self) -> BytePlace {
        self.target_link().place().unwrap_or_else(|e| refuse(e))
    }

    /// +0xC: the pointer to the choices, the edit record or the text action.
    pub fn data(self) -> u32 {
        match self.0 {
            Spot::Typed(item) => item.data.get(),
            Spot::Bytes(at) => rd(Self::field(at, 0xC, 4)),
        }
    }

    /// +0xC as the byte the least value is (a value item).
    pub fn value_least(self) -> u8 {
        match self.0 {
            Spot::Typed(item) => item.data.get().to_le_bytes()[0],
            Spot::Bytes(at) => rb(Self::field(at, 0xC, 1)),
        }
    }

    /// +0xD: the most value (a value item).
    pub fn value_most(self) -> u8 {
        match self.0 {
            Spot::Typed(item) => item.data.get().to_le_bytes()[1],
            Spot::Bytes(at) => rb(Self::field(at, 0xD, 1)),
        }
    }

    /// +0xC as the item's choices (a choice item; a link): the table is bytes of the
    /// installer's pages, read on as far as the original reads (see `Choices`).
    pub fn choices(self) -> Choices {
        Choices(
            PageLink::from_raw(self.data())
                .in_pages()
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
/// length is the item's own count, which the original does not check an index against.
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
        write!(f, "EditField(pages +{:#x})", pages_offset_of(self.0))
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

    #[test]
    fn a_pieces_item_is_kept_as_an_offset_in_the_pages() {
        let title = ListItem::of_title();
        let offset = title.pages_offset().unwrap();
        assert_eq!(ListItem::in_pages(offset).unwrap(), title);
        // The title's record is where IDA has it (0x51DD78), from the pages' first byte.
        assert_eq!(
            offset as usize,
            core::mem::offset_of!(crate::install::vars::InstallerPages, title)
        );
    }

    #[test]
    fn an_item_outside_the_pages_is_refused() {
        let size = size_of::<crate::install::vars::InstallerPages>() as u32;
        assert!(ListItem::in_pages(size - ITEM).is_ok());
        assert!(ListItem::in_pages(size - ITEM + 1).is_err());
        assert!(ListItem::in_pages(u32::MAX).is_err());
    }
}
