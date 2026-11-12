//! Where things are in `InstallerPages`: the one place that turns the pages' records (a
//! table entry, a list, a box, the edit record, the title item) into byte positions, and
//! reads the bytes that follow them.
//!
//! The original reads on past the end of what it is given: an item past a list's items is the
//! bytes that follow the list (the next list, the table, a record), a choice past a table is
//! the bytes after it, a page entry's word past its answers is whatever comes next. Those
//! reads are neighbour reads of the pages' bytes, bounded by the pages as a whole; a
//! position here names the record it starts at, and `plus` walks on from it as the original's
//! address sums do.

use crate::errors::BadInput;
use crate::install::page_links::{ListId, installer};
use crate::install::vars::{InstallerPages, InstallerPaths};
use crate::menu::list_handle::ListHeader;
use crate::place::BytePlace;
use core::mem::{offset_of, size_of};

/// A position in `InstallerPages`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PagesAt(usize);

impl core::fmt::Debug for PagesAt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::fmt::LowerHex for PagesAt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::LowerHex::fmt(&self.0, f)
    }
}

impl PagesAt {
    /// Page table entry `index`.
    pub const fn table(index: u32) -> Self {
        Self(offset_of!(InstallerPages, page_table) + 4 * index as usize)
    }

    /// The start (the header) of one of the lists.
    pub const fn list(list: ListId) -> Self {
        const fn action_page(index: usize) -> usize {
            offset_of!(InstallerPages, action_pages) + index * size_of::<ListHeader>()
        }
        Self(match list {
            ListId::Language => offset_of!(InstallerPages, language_menu),
            ListId::Main => offset_of!(InstallerPages, main_menu),
            ListId::Mode => offset_of!(InstallerPages, mode_menu),
            ListId::Drives8 => offset_of!(InstallerPages, drive_menu_8),
            ListId::Drives16 => offset_of!(InstallerPages, drive_menu_16),
            ListId::DrivesMore => offset_of!(InstallerPages, menu_more),
            ListId::Folder => offset_of!(InstallerPages, folder_menu),
            ListId::Room => offset_of!(InstallerPages, room_menu),
            ListId::Dxsetup => offset_of!(InstallerPages, dxsetup_menu),
            ListId::Finish => offset_of!(InstallerPages, finish_menu),
            ListId::InstallFiles => action_page(0),
            ListId::Uninstall => action_page(1),
            ListId::OpenInstalled => action_page(2),
            ListId::AskRegistration => action_page(3),
        })
    }

    /// Box `index` of the pages' boxes.
    pub const fn box_at(index: u32) -> Self {
        Self(offset_of!(InstallerPages, boxes) + 16 * index as usize)
    }

    /// The folder item's edit record.
    pub const fn edit() -> Self {
        Self(offset_of!(InstallerPages, folder_edit))
    }

    /// The title's item.
    pub const fn title() -> Self {
        Self(offset_of!(InstallerPages, title))
    }

    /// The byte offset (tests compare it with the layout).
    #[cfg(test)]
    pub fn offset(self) -> usize {
        self.0
    }

    /// The position at byte `offset` (tests sweep every byte).
    #[cfg(test)]
    pub fn at_offset(offset: usize) -> Self {
        Self(offset)
    }

    /// Where `thing`, a field of the installer's pages, is.
    pub fn of<T>(thing: &T) -> Self {
        let pages = &installer().pages;
        Self((std::ptr::from_ref(thing) as usize).wrapping_sub(std::ptr::from_ref(pages) as usize))
    }

    /// The position a piece keeps for its item (a dword: the item's offset in the pages), if
    /// an item's 0x10 bytes fit before the pages' end.
    pub fn of_item_word(word: u32) -> Option<Self> {
        let at = Self(word as usize);
        at.item_fits().then_some(at)
    }

    /// The dword a piece keeps for an item here.
    pub fn item_word(self) -> u32 {
        self.0 as u32
    }

    /// Whether an item's 0x10 bytes fit between here and the pages' end.
    pub fn item_fits(self) -> bool {
        self.0.saturating_add(0x10) <= size_of::<InstallerPages>()
    }

    /// `n` bytes on, wrapping at 32 bits as the original's address sum does.
    pub fn plus(self, n: u32) -> Self {
        Self((self.0 as u32).wrapping_add(n) as usize)
    }

    /// `n` bytes on without the 32-bit wrap (a page entry's neighbour word: an offset
    /// past the pages' end is read as out of them).
    pub fn plus_wide(self, n: u32) -> Self {
        Self(self.0.wrapping_add(n as usize))
    }

    /// The bytes from here, bounded by the pages as a whole.
    pub fn place(self) -> BytePlace {
        BytePlace::in_store(&installer().pages, self.0)
    }

    /// The pages' dword here (not out of the pages).
    pub fn dword(self) -> Result<u32, BadInput> {
        let mut bytes = [0; 4];
        crate::cells::load_cells(&installer().pages, self.0, &mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }
}

/// The folder typed, `offset` bytes in: a place in the installer's path buffers.
pub fn typed_folder(offset: u32) -> BytePlace {
    BytePlace::in_store(
        &installer().paths,
        offset_of!(InstallerPaths, typed) + offset as usize,
    )
}
