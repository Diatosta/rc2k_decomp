//! Where things are in `InstallerPages`: the one place that turns the pages' records (a table
//! entry, a list, a box, the edit record, the title item) into byte positions, and reads the
//! bytes that follow them.
//!
//! A read past the end of what it is given goes on into the bytes that follow (an item past a
//! list's items is the next list, a choice past a table the bytes after it), bounded by the
//! pages as a whole. A position (`PagesAt`) names the
//! record it starts at (or the field and byte, where no record starts), and `plus` walks on
//! from it.
//!
//! The pages' bytes are run on through an explicit chain of their fields in declaration order
//! (`PagesField`, `PagesChain`), not through the record's memory layout; the tests hold the
//! offsets against the old layout frozen in `NAMED_PLACES`.

use crate::cells::Store;
use crate::errors::BadInput;
use crate::install::page_links::{ListId, TABLE_LEN, installer};
use crate::install::vars::InstallerPages;
use crate::menu::list_handle::{EditRecord, Item, ListHeader};
use crate::place::{BytePlace, Chain};
use core::mem::size_of;

/// A list's header bytes (`menu::list_handle`).
const HEADER: usize = size_of::<ListHeader>();
/// An item's bytes.
const ITEM: usize = size_of::<Item>();
/// A box's bytes (`{x, y, width, height}`).
const BOX: usize = 16;
/// How many boxes the pages hold.
const BOXES: u32 = 9;

/// The fields of `InstallerPages`, in declaration order (the order the pages' bytes run on
/// in: `NAMED_PLACES` in the tests holds the frozen layout).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PagesField {
    Lead,
    Layout,
    Choice,
    ChoiceBefore,
    Repeats,
    LastKey,
    KeyTaken,
    ListAnswer,
    ListLeave,
    Gap51D417,
    KeyClock,
    FrameClock,
    MoveClock,
    Gap51D427,
    Texts,
    MsgError,
    MsgCancel,
    RemoveStrings,
    BarGlyphsList,
    Bar,
    Frame,
    Fonts,
    KindFonts,
    PageTable,
    LanguageMenu,
    MainMenu,
    ModeMenu,
    DriveMenu8,
    DriveMenu16,
    MenuMore,
    FolderMenu,
    RoomMenu,
    DxsetupMenu,
    FinishMenu,
    ActionPages,
    FolderEdit,
    Title,
    Boxes,
    BadFolderChars,
    Tail,
}

impl PagesField {
    /// Every field, in declaration order.
    pub const ALL: [PagesField; 40] = [
        PagesField::Lead,
        PagesField::Layout,
        PagesField::Choice,
        PagesField::ChoiceBefore,
        PagesField::Repeats,
        PagesField::LastKey,
        PagesField::KeyTaken,
        PagesField::ListAnswer,
        PagesField::ListLeave,
        PagesField::Gap51D417,
        PagesField::KeyClock,
        PagesField::FrameClock,
        PagesField::MoveClock,
        PagesField::Gap51D427,
        PagesField::Texts,
        PagesField::MsgError,
        PagesField::MsgCancel,
        PagesField::RemoveStrings,
        PagesField::BarGlyphsList,
        PagesField::Bar,
        PagesField::Frame,
        PagesField::Fonts,
        PagesField::KindFonts,
        PagesField::PageTable,
        PagesField::LanguageMenu,
        PagesField::MainMenu,
        PagesField::ModeMenu,
        PagesField::DriveMenu8,
        PagesField::DriveMenu16,
        PagesField::MenuMore,
        PagesField::FolderMenu,
        PagesField::RoomMenu,
        PagesField::DxsetupMenu,
        PagesField::FinishMenu,
        PagesField::ActionPages,
        PagesField::FolderEdit,
        PagesField::Title,
        PagesField::Boxes,
        PagesField::BadFolderChars,
        PagesField::Tail,
    ];

    fn bytes(self, pages: &'static InstallerPages) -> &'static dyn Store {
        match self {
            PagesField::Lead => &pages._lead,
            PagesField::Layout => &pages.layout,
            PagesField::Choice => &pages.choice,
            PagesField::ChoiceBefore => &pages.choice_before,
            PagesField::Repeats => &pages.repeats,
            PagesField::LastKey => &pages.last_key,
            PagesField::KeyTaken => &pages.key_taken,
            PagesField::ListAnswer => &pages.list_answer,
            PagesField::ListLeave => &pages.list_leave,
            PagesField::Gap51D417 => &pages._gap_51d417,
            PagesField::KeyClock => &pages.key_clock,
            PagesField::FrameClock => &pages.frame_clock,
            PagesField::MoveClock => &pages.move_clock,
            PagesField::Gap51D427 => &pages._gap_51d427,
            PagesField::Texts => &pages.texts,
            PagesField::MsgError => &pages.msg_error,
            PagesField::MsgCancel => &pages.msg_cancel,
            PagesField::RemoveStrings => &pages.remove_strings,
            PagesField::BarGlyphsList => &pages.bar_glyphs_list,
            PagesField::Bar => &pages.bar,
            PagesField::Frame => &pages.frame,
            PagesField::Fonts => &pages.fonts,
            PagesField::KindFonts => &pages.kind_fonts,
            PagesField::PageTable => &pages.page_table,
            PagesField::LanguageMenu => &pages.language_menu,
            PagesField::MainMenu => &pages.main_menu,
            PagesField::ModeMenu => &pages.mode_menu,
            PagesField::DriveMenu8 => &pages.drive_menu_8,
            PagesField::DriveMenu16 => &pages.drive_menu_16,
            PagesField::MenuMore => &pages.menu_more,
            PagesField::FolderMenu => &pages.folder_menu,
            PagesField::RoomMenu => &pages.room_menu,
            PagesField::DxsetupMenu => &pages.dxsetup_menu,
            PagesField::FinishMenu => &pages.finish_menu,
            PagesField::ActionPages => &pages.action_pages,
            PagesField::FolderEdit => &pages.folder_edit,
            PagesField::Title => &pages.title,
            PagesField::Boxes => &pages.boxes,
            PagesField::BadFolderChars => &pages.bad_folder_chars,
            PagesField::Tail => &pages._tail,
        }
    }

    /// Where each field starts in the run of the pages' bytes (the fields back to back in
    /// declaration order), and the run's end.
    fn starts() -> &'static [usize; 41] {
        static STARTS: std::sync::OnceLock<[usize; 41]> = std::sync::OnceLock::new();
        STARTS.get_or_init(|| {
            let pages = &installer().pages;
            let mut starts = [0; 41];
            for (n, field) in Self::ALL.into_iter().enumerate() {
                starts[n + 1] = starts[n] + field.bytes(pages).size();
            }
            starts
        })
    }

    /// The field's first byte in the run of the pages' bytes.
    fn offset(self) -> usize {
        Self::starts()[self as usize]
    }

    fn len(self) -> usize {
        Self::starts()[self as usize + 1] - self.offset()
    }

    fn total() -> usize {
        Self::starts()[Self::ALL.len()]
    }

    /// The field holding byte `offset` of the pages and the byte's place in it, if inside.
    fn holding(offset: usize) -> Option<(PagesField, usize)> {
        let at = Self::ALL.partition_point(|field| field.offset() <= offset);
        (offset < Self::total() && at > 0).then(|| {
            let field = Self::ALL[at - 1];
            (field, offset - field.offset())
        })
    }

    /// The list a list field holds (the action pages hold four headers: see `PagesAt::list`).
    fn list(self) -> Option<ListId> {
        Some(match self {
            PagesField::LanguageMenu => ListId::Language,
            PagesField::MainMenu => ListId::Main,
            PagesField::ModeMenu => ListId::Mode,
            PagesField::DriveMenu8 => ListId::Drives8,
            PagesField::DriveMenu16 => ListId::Drives16,
            PagesField::MenuMore => ListId::DrivesMore,
            PagesField::FolderMenu => ListId::Folder,
            PagesField::RoomMenu => ListId::Room,
            PagesField::DxsetupMenu => ListId::Dxsetup,
            PagesField::FinishMenu => ListId::Finish,
            _ => return None,
        })
    }
}

/// Where one of the pages' lists is: the field and the header's first byte in it.
fn list_place(list: ListId) -> (PagesField, usize) {
    let action_page = |index: usize| (PagesField::ActionPages, index * HEADER);
    match list {
        ListId::Language => (PagesField::LanguageMenu, 0),
        ListId::Main => (PagesField::MainMenu, 0),
        ListId::Mode => (PagesField::ModeMenu, 0),
        ListId::Drives8 => (PagesField::DriveMenu8, 0),
        ListId::Drives16 => (PagesField::DriveMenu16, 0),
        ListId::DrivesMore => (PagesField::MenuMore, 0),
        ListId::Folder => (PagesField::FolderMenu, 0),
        ListId::Room => (PagesField::RoomMenu, 0),
        ListId::Dxsetup => (PagesField::DxsetupMenu, 0),
        ListId::Finish => (PagesField::FinishMenu, 0),
        ListId::InstallFiles => action_page(0),
        ListId::Uninstall => action_page(1),
        ListId::OpenInstalled => action_page(2),
        ListId::AskRegistration => action_page(3),
    }
}

/// A position in `InstallerPages`, by the record it names. Every position has one form: a
/// record's start is named by the record (a table entry, a list's header, a typed item, a
/// box, the edit record, the title), any other byte of the pages by its field, and a byte
/// past the pages by how far past their end it is.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PagesAt {
    /// Page table entry `.0` (below `TABLE_LEN`).
    Table(u32),
    /// The header of one of the lists.
    List(ListId),
    /// Typed item `.1` of a list (below its items).
    Item(ListId, u32),
    /// Box `.0` of the pages' boxes (below 9).
    Box(u32),
    /// The folder item's edit record.
    Edit,
    /// The title's item.
    Title,
    /// Byte `.1` of a field, where none of the records above starts.
    Field(PagesField, usize),
    /// `.0` bytes past the pages' end.
    PastEnd(usize),
}

impl core::fmt::Debug for PagesAt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.run_offset())
    }
}

impl core::fmt::LowerHex for PagesAt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::LowerHex::fmt(&self.run_offset(), f)
    }
}

impl PagesAt {
    pub fn table(index: u32) -> Self {
        assert!(index < TABLE_LEN, "a page table entry past the table");
        Self::Table(index)
    }

    /// The start (the header) of one of the lists.
    pub fn list(list: ListId) -> Self {
        Self::List(list)
    }

    pub fn box_at(index: u32) -> Self {
        assert!(index < BOXES, "a box past the pages' boxes");
        Self::Box(index)
    }

    pub fn edit() -> Self {
        Self::Edit
    }

    pub fn title() -> Self {
        Self::Title
    }

    /// The page table index of the entry holding this byte, if a table entry does.
    pub fn table_index(self) -> Option<u32> {
        match self {
            Self::Table(index) => Some(index),
            Self::Field(PagesField::PageTable, byte) => Some((byte / 4) as u32),
            _ => None,
        }
    }

    /// The position of byte `offset` of the pages, by the record it names.
    fn from_offset(offset: usize) -> Self {
        let Some((field, byte)) = PagesField::holding(offset) else {
            return Self::PastEnd(offset - PagesField::total());
        };
        let named = match field {
            PagesField::PageTable if byte.is_multiple_of(4) => Some(Self::Table((byte / 4) as u32)),
            PagesField::ActionPages if byte.is_multiple_of(HEADER) => Some(Self::List(
                ListId::ALL[ListId::ALL.len() - 4 + byte / HEADER],
            )),
            PagesField::FolderEdit if byte == 0 => Some(Self::Edit),
            PagesField::Title if byte == 0 => Some(Self::Title),
            PagesField::Boxes if byte.is_multiple_of(BOX) => Some(Self::Box((byte / BOX) as u32)),
            _ => field.list().and_then(|list| match byte {
                0 => Some(Self::List(list)),
                _ if byte >= HEADER && (byte - HEADER).is_multiple_of(ITEM) => {
                    Some(Self::Item(list, ((byte - HEADER) / ITEM) as u32))
                }
                _ => None,
            }),
        };
        named.unwrap_or(Self::Field(field, byte))
    }

    /// The byte offset in the pages' run (the tests and the golden name positions by it).
    #[cfg(test)]
    pub fn offset(self) -> usize {
        self.run_offset()
    }

    /// The byte offset in the pages' run: where the chain reads, and the dword a piece keeps
    /// for an item (`item_word`).
    fn run_offset(self) -> usize {
        let field_byte = |(field, byte): (PagesField, usize)| field.offset() + byte;
        match self {
            Self::Table(index) => field_byte((PagesField::PageTable, 4 * index as usize)),
            Self::List(list) => field_byte(list_place(list)),
            Self::Item(list, n) => field_byte(list_place(list)) + HEADER + ITEM * n as usize,
            Self::Box(index) => field_byte((PagesField::Boxes, BOX * index as usize)),
            Self::Edit => field_byte((PagesField::FolderEdit, 0)),
            Self::Title => field_byte((PagesField::Title, 0)),
            Self::Field(field, byte) => field_byte((field, byte)),
            Self::PastEnd(past) => PagesField::total().wrapping_add(past),
        }
    }

    #[cfg(test)]
    pub fn at_offset(offset: usize) -> Self {
        Self::from_offset(offset)
    }

    /// Where `header`, one of the lists' headers, is.
    pub fn of_header(header: &ListHeader) -> Self {
        ListId::ALL
            .into_iter()
            .find(|&list| core::ptr::eq(crate::menu::list_handle::list_header(list), header))
            .map(Self::List)
            .unwrap_or_else(|| {
                crate::install::installer::refuse(BadInput(
                    "a list header that is none of the installer's lists",
                ))
            })
    }

    /// Where `item`, one of the lists' typed items or the title, is.
    pub fn of_item(item: &Item) -> Self {
        if core::ptr::eq(item, &raw const installer().pages.title) {
            return Self::Title;
        }
        for list in ListId::ALL {
            let items = crate::menu::list_handle::list_items(list);
            if let Some(n) = items.iter().position(|typed| core::ptr::eq(typed, item)) {
                return Self::Item(list, n as u32);
            }
        }
        crate::install::installer::refuse(BadInput("an item that is none of the installer's items"))
    }

    pub fn of_edit(edit: &EditRecord) -> Self {
        if !core::ptr::eq(edit, &raw const installer().pages.folder_edit) {
            crate::install::installer::refuse(BadInput(
                "an edit record that is not the installer's",
            ));
        }
        Self::Edit
    }

    /// The position a piece keeps for its item (a dword: the item's offset in the pages), if
    /// an item's 0x10 bytes fit before the pages' end.
    pub fn of_item_word(word: u32) -> Option<Self> {
        let at = Self::from_offset(word as usize);
        at.item_fits().then_some(at)
    }

    /// The dword a piece keeps for an item here.
    pub fn item_word(self) -> u32 {
        self.run_offset() as u32
    }

    /// Whether an item's 0x10 bytes fit between here and the pages' end.
    pub fn item_fits(self) -> bool {
        self.run_offset().saturating_add(ITEM) <= PagesField::total()
    }

    /// `n` bytes on, wrapping at 32 bits.
    pub fn plus(self, n: u32) -> Self {
        Self::from_offset((self.run_offset() as u32).wrapping_add(n) as usize)
    }

    /// `n` bytes on without the 32-bit wrap (a page entry's neighbour word: an offset
    /// past the pages' end is read as out of them).
    pub fn plus_wide(self, n: u32) -> Self {
        Self::from_offset(self.run_offset().wrapping_add(n as usize))
    }

    /// The bytes from here, bounded by the pages as a whole (run on from field to field).
    pub fn place(self) -> BytePlace {
        BytePlace::Chain(&PagesChain, self.run_offset())
    }

    /// The pages' dword here (not out of the pages), across the fields' edges.
    pub fn dword(self) -> Result<u32, BadInput> {
        let mut bytes = [0; 4];
        PagesChain.load(self.run_offset(), &mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }
}

/// The installer's pages as one run of bytes: their fields in declaration order, back to
/// back. A read or write that runs on past a field goes on into the next one; the pages'
/// memory layout is not used.
pub struct PagesChain;

impl PagesChain {
    /// Each field's part of `len` bytes from `offset` (the part's place in the field, and in
    /// the bytes), or refused past the pages' end.
    fn parts(
        offset: usize,
        len: usize,
    ) -> Result<impl Iterator<Item = (PagesField, usize, core::ops::Range<usize>)>, BadInput> {
        let end = offset
            .checked_add(len)
            .filter(|&end| end <= PagesField::total())
            .ok_or(BadInput("past the end of a record"))?;
        let mut at = offset;
        Ok(core::iter::from_fn(move || {
            let (field, byte) = PagesField::holding(at).filter(|_| at < end)?;
            let upto = end.min(field.offset() + field.len());
            let part = (field, byte, at - offset..upto - offset);
            at = upto;
            Some(part)
        }))
    }
}

impl crate::place::Chain for PagesChain {
    fn size(&self) -> usize {
        PagesField::total()
    }

    fn load(&self, offset: usize, to: &mut [u8]) -> Result<(), BadInput> {
        let pages = &installer().pages;
        for (field, byte, part) in Self::parts(offset, to.len())? {
            field.bytes(pages).load(byte, &mut to[part])?;
        }
        Ok(())
    }

    fn store(&self, offset: usize, from: &[u8]) -> Result<(), BadInput> {
        let pages = &installer().pages;
        for (field, byte, part) in Self::parts(offset, from.len())? {
            field.bytes(pages).store(byte, &from[part])?;
        }
        Ok(())
    }

    fn byte_at(&self, offset: usize) -> *const u8 {
        let pages = &installer().pages;
        match PagesField::holding(offset) {
            Some((field, byte)) => field.bytes(pages).first().wrapping_add(byte),
            // Past the pages: the memory after the whole record, which no field holds.
            None => core::ptr::from_ref(pages)
                .cast::<u8>()
                .wrapping_add(size_of::<InstallerPages>())
                .wrapping_add(offset.wrapping_sub(PagesField::total())),
        }
    }
}

/// The folder typed, `offset` bytes in: a place in the installer's path buffer for it.
pub fn typed_folder(offset: u32) -> BytePlace {
    BytePlace::In(&installer().paths.typed, offset as usize)
}

/// The pages' named places as the old layout had them, frozen: every field of `InstallerPages`
/// in declaration order, its byte offset and its length. The oracle the run-on chain and the
/// handles are proven against.
#[cfg(test)]
pub(crate) const NAMED_PLACES: [(&str, usize, usize); 40] = [
    ("_lead", 0x0, 7),
    ("layout", 0x7, 4096),
    ("choice", 0x1007, 4),
    ("choice_before", 0x100B, 4),
    ("repeats", 0x100F, 4),
    ("last_key", 0x1013, 1),
    ("key_taken", 0x1014, 1),
    ("list_answer", 0x1015, 1),
    ("list_leave", 0x1016, 1),
    ("_gap_51d417", 0x1017, 4),
    ("key_clock", 0x101B, 4),
    ("frame_clock", 0x101F, 4),
    ("move_clock", 0x1023, 4),
    ("_gap_51d427", 0x1027, 12),
    ("texts", 0x1033, 0x20),
    ("msg_error", 0x1053, 0x10),
    ("msg_cancel", 0x1063, 0x10),
    ("remove_strings", 0x1073, 0x10),
    ("bar_glyphs_list", 0x1083, 0x20),
    ("bar", 0x10A3, 16),
    ("frame", 0x10B3, 17),
    ("fonts", 0x10C4, 100),
    ("kind_fonts", 0x1128, 108),
    ("page_table", 0x1194, 220),
    ("language_menu", 0x1270, 0xB8),
    ("main_menu", 0x1328, 0x68),
    ("mode_menu", 0x1390, 0x68),
    ("drive_menu_8", 0x13F8, 0xB8),
    ("drive_menu_16", 0x14B0, 0x138),
    ("menu_more", 0x15E8, 0x1B8),
    ("folder_menu", 0x17A0, 0x78),
    ("room_menu", 0x1818, 0x48),
    ("dxsetup_menu", 0x1860, 0x58),
    ("finish_menu", 0x18B8, 0x58),
    ("action_pages", 0x1910, 0x60),
    ("folder_edit", 0x1970, 8),
    ("title", 0x1978, 0x10),
    ("boxes", 0x1988, 0x90),
    ("bad_folder_chars", 0x1A18, 20),
    ("_tail", 0x1A2C, 4),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::list_handle::ListMenu;

    /// The frozen table is the record's fields: each field's name and length (`size_of_val`)
    /// in declaration order, the fields back to back from 0 to the old 0x1A30 bytes.
    #[test]
    fn the_named_places_are_the_fields() {
        let _lock = crate::test_support::test_lock();
        let pages = &installer().pages;
        macro_rules! fields {
            ($($field:ident),* $(,)?) => {
                [$((stringify!($field), core::mem::size_of_val(&pages.$field))),*]
            };
        }
        let fields = fields!(
            _lead,
            layout,
            choice,
            choice_before,
            repeats,
            last_key,
            key_taken,
            list_answer,
            list_leave,
            _gap_51d417,
            key_clock,
            frame_clock,
            move_clock,
            _gap_51d427,
            texts,
            msg_error,
            msg_cancel,
            remove_strings,
            bar_glyphs_list,
            bar,
            frame,
            fonts,
            kind_fonts,
            page_table,
            language_menu,
            main_menu,
            mode_menu,
            drive_menu_8,
            drive_menu_16,
            menu_more,
            folder_menu,
            room_menu,
            dxsetup_menu,
            finish_menu,
            action_pages,
            folder_edit,
            title,
            boxes,
            bad_folder_chars,
            _tail,
        );
        let mut next = 0;
        for ((name, len), (named, offset, named_len)) in fields.into_iter().zip(NAMED_PLACES) {
            assert_eq!((name, len), (named, named_len));
            assert_eq!(offset, next, "{name} follows the field before it");
            next = offset + len;
        }
        assert_eq!(next, 0x1A30);
    }

    /// The run-on chain lays the fields out as the frozen table does: each field where the
    /// table puts it, as long, in the same order.
    #[test]
    fn the_chain_is_the_named_places() {
        let _lock = crate::test_support::test_lock();
        for (field, (name, offset, len)) in PagesField::ALL.into_iter().zip(NAMED_PLACES) {
            assert_eq!((field.offset(), field.len()), (offset, len), "{name}");
        }
        assert_eq!(PagesField::total(), 0x1A30);
        // Every byte is named by the record or field that holds it, and back.
        for offset in 0..PagesField::total() + 0x20 {
            assert_eq!(PagesAt::at_offset(offset).offset(), offset);
        }
    }

    /// A read through the chain is the read of the fields' own bytes laid back to back, at
    /// every byte and length, refusals included; a write through it writes the fields' bytes.
    #[test]
    fn the_chain_reads_the_fields_bytes() {
        let _lock = crate::test_support::test_lock();
        let pages = &installer().pages;
        let size = PagesField::total();
        // The run as each field's own store gives its bytes.
        let mut run = Vec::new();
        for field in PagesField::ALL {
            let mut bytes = vec![0; field.len()];
            field.bytes(pages).load(0, &mut bytes).unwrap();
            run.extend(bytes);
        }
        assert_eq!(run.len(), size);
        for len in [0, 1, 2, 3, 4, 7, 0x10, 0x30] {
            for offset in (0..size + 8).chain([usize::MAX - 2, usize::MAX]) {
                let mut by_chain = vec![0xAA; len];
                let chain = PagesChain.load(offset, &mut by_chain);
                match offset.checked_add(len).filter(|&end| end <= size) {
                    Some(end) => {
                        assert_eq!(chain, Ok(()), "{len} bytes at {offset}");
                        assert_eq!(by_chain, run[offset..end], "{len} bytes at {offset}");
                    }
                    None => assert!(chain.is_err(), "{len} bytes at {offset}"),
                }
            }
        }
        // A write across a field's edge (the gap before the clocks into the key's clock),
        // put back after.
        let at = PagesField::KeyClock.offset() - 2;
        let mut saved = [0; 4];
        PagesChain.load(at, &mut saved).unwrap();
        let clock = pages.key_clock.get();
        PagesChain.store(at, &[1, 2, 3, 4]).unwrap();
        let written = (
            pages._gap_51d417.get(2),
            pages._gap_51d417.get(3),
            pages.key_clock.get(),
        );
        PagesChain.store(at, &saved).unwrap();
        assert_eq!(written, (1, 2, (clock & !0xFFFF) | 0x0403));
        assert!(PagesChain.store(size - 1, &[0, 0]).is_err());
        // A place compares by the byte of memory that holds it: each byte its field's, no two
        // alike, and past the pages none of a field's.
        let mut seen = std::collections::HashSet::new();
        for offset in 0..size {
            let (field, byte) = PagesField::holding(offset).unwrap();
            let at = PagesChain.byte_at(offset);
            assert_eq!(at, field.bytes(pages).first().wrapping_add(byte));
            assert!(seen.insert(at), "{offset}");
        }
        for past in 0..0x20 {
            assert!(!seen.contains(&PagesChain.byte_at(size + past)), "{past}");
        }
    }

    /// The handles start where the frozen table says their records are.
    #[test]
    fn the_handles_start_at_the_named_places() {
        let _lock = crate::test_support::test_lock();
        let place = |name: &str| NAMED_PLACES.iter().find(|(n, _, _)| *n == name).unwrap().1;
        for index in 0..crate::install::page_links::TABLE_LEN {
            let at = PagesAt::table(index);
            assert_eq!(at.offset(), place("page_table") + 4 * index as usize);
            assert_eq!(at.table_index(), Some(index));
        }
        let lists = [
            (ListId::Language, place("language_menu")),
            (ListId::Main, place("main_menu")),
            (ListId::Mode, place("mode_menu")),
            (ListId::Drives8, place("drive_menu_8")),
            (ListId::Drives16, place("drive_menu_16")),
            (ListId::DrivesMore, place("menu_more")),
            (ListId::Folder, place("folder_menu")),
            (ListId::Room, place("room_menu")),
            (ListId::Dxsetup, place("dxsetup_menu")),
            (ListId::Finish, place("finish_menu")),
            (ListId::InstallFiles, place("action_pages")),
            (ListId::Uninstall, place("action_pages") + 0x18),
            (ListId::OpenInstalled, place("action_pages") + 0x30),
            (ListId::AskRegistration, place("action_pages") + 0x48),
        ];
        for (list, offset) in lists {
            assert_eq!(PagesAt::list(list).offset(), offset, "{list:?}");
        }
        for index in 0..9 {
            assert_eq!(
                PagesAt::box_at(index).offset(),
                place("boxes") + 16 * index as usize
            );
        }
        assert_eq!(PagesAt::edit().offset(), place("folder_edit"));
        assert_eq!(PagesAt::title().offset(), place("title"));
    }

    /// What an item read past a list's items is: the header of the list that follows in the
    /// pages (the last action page's is the folder item's edit record). A read that runs on
    /// reaches that neighbour (`ListMenu::item`).
    #[test]
    fn an_item_past_a_list_is_the_field_after_it() {
        let _lock = crate::test_support::test_lock();
        let follows = [
            (ListId::Language, PagesAt::list(ListId::Main)),
            (ListId::Main, PagesAt::list(ListId::Mode)),
            (ListId::Mode, PagesAt::list(ListId::Drives8)),
            (ListId::Drives8, PagesAt::list(ListId::Drives16)),
            (ListId::Drives16, PagesAt::list(ListId::DrivesMore)),
            (ListId::DrivesMore, PagesAt::list(ListId::Folder)),
            (ListId::Folder, PagesAt::list(ListId::Room)),
            (ListId::Room, PagesAt::list(ListId::Dxsetup)),
            (ListId::Dxsetup, PagesAt::list(ListId::Finish)),
            (ListId::Finish, PagesAt::list(ListId::InstallFiles)),
            (ListId::InstallFiles, PagesAt::list(ListId::Uninstall)),
            (ListId::Uninstall, PagesAt::list(ListId::OpenInstalled)),
            (
                ListId::OpenInstalled,
                PagesAt::list(ListId::AskRegistration),
            ),
            (ListId::AskRegistration, PagesAt::edit()),
        ];
        for (list, after) in follows {
            let menu = ListMenu::installer(list);
            let items = match list {
                ListId::Language | ListId::Drives8 => 10,
                ListId::Main | ListId::Mode => 5,
                ListId::Drives16 => 18,
                ListId::DrivesMore => 26,
                ListId::Folder => 6,
                ListId::Room => 3,
                ListId::Dxsetup | ListId::Finish => 4,
                _ => 0,
            };
            assert_eq!(
                menu.item(items).pages_offset().unwrap(),
                after.item_word(),
                "{list:?}"
            );
        }
    }

    /// M4 for the choice items: no item of the installer's lists is a choice item (kind 2),
    /// and the only run-time write of an item's kind is the main page's first item, made
    /// kind 5 (`check_at_start::set_script_entry`). A choice table and a choice past it are
    /// therefore never read from the pages' data; the readers stay for the generic item keys.
    /// The lists' counts hold in their items (the drive lists' two more than the drives, up
    /// to 26, and the languages up to 9 of the language list's 10).
    #[test]
    fn the_installer_has_no_choice_items() {
        let _lock = crate::test_support::test_lock();
        let mut kinds = std::collections::BTreeSet::new();
        for list in ListId::ALL {
            let menu = ListMenu::installer(list);
            for n in 0..u32::from(menu.count()) {
                kinds.insert(menu.item(n).kind());
            }
        }
        assert!(!kinds.contains(&2), "kinds {kinds:?}");
    }
}
