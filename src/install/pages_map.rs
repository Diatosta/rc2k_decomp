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
use crate::install::vars::InstallerPages;
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

    /// The page table index of the entry that starts here, if a table entry does.
    pub fn table_index(self) -> Option<u32> {
        let first = offset_of!(InstallerPages, page_table);
        let past = first + 4 * crate::install::page_links::TABLE_LEN as usize;
        (first..past)
            .contains(&self.0)
            .then(|| ((self.0 - first) / 4) as u32)
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

/// The folder typed, `offset` bytes in: a place in the installer's path buffer for it.
pub fn typed_folder(offset: u32) -> BytePlace {
    BytePlace::In(&installer().paths.typed, offset as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::list_handle::ListMenu;

    /// What an item read past a list's items is: the header of the list that follows in the
    /// pages (the last action page's is the folder item's edit record). A read that runs on
    /// reaches that neighbour, as the original's (`ListMenu::item`).
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
