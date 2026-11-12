//! The links in the installer pages' data, as handles.
//!
//! The original's pages hold addresses: a page entry names its list, an item its box, its
//! choices, its edit record, its value or its text, a list its neighbours table, a drawing
//! list's record its font or palette. The data here holds a handle in each of those dwords,
//! made once where the data is built (`vars::InstallerPages::new`, from IDA's address), and
//! every reader resolves it with bounds. A handle is a kind in the high byte and a number
//! below:
//!
//! - `TABLE`: a dword of `InstallerPages::page_table` (the page entries), by index;
//! - `LIST`: one of the pages' lists, by `ListId` (a page entry's first word names a list, or one of
//!   the header-only `action_pages`);
//! - `BOX`: one of `InstallerPages::boxes`, by index (0 the title item's, 1 the title's);
//! - `EDIT`: the folder item's edit record;
//! - `FONT`: a font of the font block, by number (`gfx::typefaces::FontId`);
//! - `BLENDED_PALETTE`: the blended palette (the install bar's glyphs);
//! - `TYPED_PATH`: a byte offset in `InstallerPaths::typed` (the folder item's text starts
//!   past the drive's `c:`).
//!
//! 0 is no link (a page with no next entry, the title's box). A thing of the pages resolves
//! to its place in `InstallerPages` (`in_pages`), which is bounded by the whole of the
//! pages: a read that runs on past a list's items or a table's end reads the bytes that
//! follow, as the original's.

use crate::errors::BadInput;
use crate::gfx::typefaces::{self, FontId};
use crate::install::vars::{InstallerPages, InstallerPaths};
use crate::menu::list_handle::EditRecord;
use crate::place::BytePlace;

const KIND_SHIFT: u32 = 24;
const NUMBER: u32 = (1 << KIND_SHIFT) - 1;
const TABLE: u32 = 1;
const FONT: u32 = 2;
const BLENDED_PALETTE: u32 = 3;
const TYPED_PATH: u32 = 4;
const LIST: u32 = 5;
const BOX: u32 = 6;
const EDIT: u32 = 8;

/// IDA's address of `InstallerPaths::typed` (`InstallerPaths.typed`): the folder typed on the page. The
/// folder item's text starts two bytes in (0x95EBFA, past the drive's `c:`), or at its start
/// on Unix (no drive).
const TYPED_IDA: u32 = 0x95EBF8;
/// IDA's address of the blended palette the install bar's glyphs name.
const PALETTE_IDA: u32 = 0x4F3B74;

/// IDA's address of the first dword of `InstallerPages::page_table`, and how many it holds.
const TABLE_IDA: u32 = 0x51D594;
pub const TABLE_LEN: u32 = 55;
/// IDA's address of the folder item's edit record.
const EDIT_IDA: u32 = 0x51DD70;
/// IDA's address of the first of `InstallerPages::boxes` (16 bytes each, 9 of them).
const BOX_IDA: u32 = 0x51DD88;
const BOXES: u32 = 9;

/// One of the installer pages' lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListId {
    Language,
    Main,
    Mode,
    Drives8,
    Drives16,
    DrivesMore,
    Folder,
    Room,
    Dxsetup,
    Finish,
    /// The pages that are only an action (a header with no items): copying the files,
    /// uninstalling, opening the installed copy, asking to register.
    InstallFiles,
    Uninstall,
    OpenInstalled,
    AskRegistration,
}

impl ListId {
    /// The lists with IDA's address of each, in the order of the enum.
    pub(crate) const ALL: [(ListId, u32); 14] = [
        (ListId::Language, 0x51D670),
        (ListId::Main, 0x51D728),
        (ListId::Mode, 0x51D790),
        (ListId::Drives8, 0x51D7F8),
        (ListId::Drives16, 0x51D8B0),
        (ListId::DrivesMore, 0x51D9E8),
        (ListId::Folder, 0x51DBA0),
        (ListId::Room, 0x51DC18),
        (ListId::Dxsetup, 0x51DC60),
        (ListId::Finish, 0x51DCB8),
        (ListId::InstallFiles, 0x51DD10),
        (ListId::Uninstall, 0x51DD28),
        (ListId::OpenInstalled, 0x51DD40),
        (ListId::AskRegistration, 0x51DD58),
    ];

    /// The list's byte offset in `InstallerPages`.
    pub const fn offset(self) -> usize {
        use core::mem::offset_of;
        match self {
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
            ListId::InstallFiles => offset_of!(InstallerPages, action_pages),
            ListId::Uninstall => offset_of!(InstallerPages, action_pages) + 24,
            ListId::OpenInstalled => offset_of!(InstallerPages, action_pages) + 48,
            ListId::AskRegistration => offset_of!(InstallerPages, action_pages) + 72,
        }
    }

    const fn number(self) -> u32 {
        self as u32
    }

    const fn from_number(number: u32) -> Option<Self> {
        if number < Self::ALL.len() as u32 {
            Some(Self::ALL[number as usize].0)
        } else {
            None
        }
    }

    const fn of_ida(ida: u32) -> Option<Self> {
        let mut k = 0;
        while k < Self::ALL.len() {
            if Self::ALL[k].1 == ida {
                return Some(Self::ALL[k].0);
            }
            k += 1;
        }
        None
    }
}

/// A link of the installer pages' data (a handle; see the module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageLink(u32);

impl PageLink {
    /// The link the original's address `ida` is (where the data is built; an address that
    /// names none of the kinds fails to compile).
    pub const fn of_ida(ida: u32) -> Self {
        if ida >= TABLE_IDA
            && ida < TABLE_IDA + 4 * TABLE_LEN
            && (ida - TABLE_IDA).is_multiple_of(4)
        {
            return Self((TABLE << KIND_SHIFT) | ((ida - TABLE_IDA) / 4));
        }
        if let Some(list) = ListId::of_ida(ida) {
            return Self::list(list);
        }
        if ida == EDIT_IDA {
            return Self(EDIT << KIND_SHIFT);
        }
        if ida >= BOX_IDA && ida < BOX_IDA + 16 * BOXES && (ida - BOX_IDA).is_multiple_of(16) {
            return Self::box_at((ida - BOX_IDA) / 16);
        }
        let fonts = typefaces::START as u32;
        if ida >= fonts && ida < fonts + typefaces::LEN as u32 && (ida - fonts).is_multiple_of(0x20)
        {
            return Self((FONT << KIND_SHIFT) | ((ida - fonts) / 0x20));
        }
        if ida == PALETTE_IDA {
            return Self(BLENDED_PALETTE << KIND_SHIFT);
        }
        if ida >= TYPED_IDA && ida < TYPED_IDA + 0x100 {
            return Self((TYPED_PATH << KIND_SHIFT) | (ida - TYPED_IDA));
        }
        panic!("a page link that names nothing known");
    }

    /// The link to page entry `index` of the pages' table.
    pub const fn table(index: u32) -> Self {
        assert!(index < TABLE_LEN);
        Self((TABLE << KIND_SHIFT) | index)
    }

    /// The link to one of the pages' lists.
    pub const fn list(list: ListId) -> Self {
        Self((LIST << KIND_SHIFT) | list.number())
    }

    /// The link to box `index` of the pages' boxes (0 the title item's, 1 the title's).
    pub const fn box_at(index: u32) -> Self {
        assert!(index < BOXES);
        Self((BOX << KIND_SHIFT) | index)
    }

    /// A link as the pages' data holds it.
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// The pages' dword for this link.
    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn is_none(self) -> bool {
        self.0 == 0
    }

    fn kind(self) -> u32 {
        self.0 >> KIND_SHIFT
    }

    fn number(self) -> u32 {
        self.0 & NUMBER
    }

    /// Where the thing of the installer pages the link names starts, as a byte offset in
    /// `InstallerPages`.
    pub fn pages_offset(self) -> Option<usize> {
        use core::mem::offset_of;
        let number = self.number();
        match self.kind() {
            TABLE if number < TABLE_LEN => {
                Some(offset_of!(InstallerPages, page_table) + 4 * number as usize)
            }
            LIST => ListId::from_number(number).map(ListId::offset),
            BOX if number < BOXES => Some(offset_of!(InstallerPages, boxes) + 16 * number as usize),
            EDIT if number == 0 => Some(offset_of!(InstallerPages, folder_edit)),
            _ => None,
        }
    }

    /// The place in the installer pages the link names (a table entry, a list, a box, a
    /// handler or the edit record), bounded by the pages (a table or text may run on inside
    /// them as the original's does).
    pub fn in_pages(self) -> Result<BytePlace, BadInput> {
        let install = crate::install::owner::install();
        Ok(BytePlace::in_store(&install.pages, self.pages_start()?))
    }

    /// The byte offset in the pages of the place the link names.
    pub fn pages_start(self) -> Result<usize, BadInput> {
        self.pages_offset().ok_or(BadInput(
            "a page link that is no place in the installer's pages",
        ))
    }

    /// The byte offset in the pages of the box a link names.
    pub fn box_offset(self) -> Result<usize, BadInput> {
        match self.pages_offset() {
            Some(offset) if self.kind() == BOX => Ok(offset),
            _ => Err(BadInput("a page link that is no box")),
        }
    }

    /// The pages' dword at the byte offset `at` (a table or text may run on inside them as the
    /// original's does, but not out of them).
    pub fn pages_dword(at: usize) -> Result<u32, BadInput> {
        let mut bytes = [0; 4];
        crate::pointers::load_cells(&crate::install::owner::install().pages, at, &mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }

    /// The list a link names, if it is a link to one of the pages' lists.
    pub fn list_id(self) -> Option<ListId> {
        match self.kind() {
            LIST => ListId::from_number(self.number()),
            _ => None,
        }
    }

    /// The edit record a link names (the folder item's).
    pub fn edit_record(self) -> Result<&'static EditRecord, BadInput> {
        match self.kind() {
            EDIT if self.number() == 0 => Ok(&crate::install::owner::install().pages.folder_edit),
            _ => Err(BadInput("a page link that is no edit record")),
        }
    }

    /// The place a text or a byte link names: in the pages, or the folder typed (bounded by
    /// the installer's path buffers).
    pub fn place(self) -> Result<BytePlace, BadInput> {
        let install = crate::install::owner::install();
        if self.kind() == TYPED_PATH && self.number() < 0x100 {
            let typed = core::mem::offset_of!(InstallerPaths, typed);
            return Ok(BytePlace::in_store(
                &install.paths,
                typed + self.number() as usize,
            ));
        }
        self.in_pages()
    }

    /// The font the link names.
    pub fn font(self) -> Result<FontId, BadInput> {
        let at = typefaces::START + 0x20 * self.number() as usize;
        match FontId::try_at(at) {
            Some(font) if self.kind() == FONT => Ok(font),
            _ => Err(BadInput("a page link that is no font")),
        }
    }

    /// Whether the link names the blended palette (the only palette the pages name).
    pub fn is_blended_palette(self) -> bool {
        self.0 == BLENDED_PALETTE << KIND_SHIFT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The original's address of the list `list`.
    fn list_ida(list: ListId) -> u32 {
        ListId::ALL.iter().find(|e| e.0 == list).unwrap().1
    }

    #[test]
    fn the_original_addresses_become_the_handles_of_their_kind() {
        let title_box = PageLink::of_ida(BOX_IDA + 16);
        assert_eq!(title_box, PageLink::box_at(1));
        assert_eq!(
            title_box.box_offset().unwrap(),
            core::mem::offset_of!(InstallerPages, boxes) + 16
        );
        assert_eq!(
            PageLink::of_ida(TABLE_IDA + 4 * 25),
            PageLink::table(25),
            "the drive page's entry"
        );
        assert_eq!(
            PageLink::of_ida(list_ida(ListId::DrivesMore)),
            PageLink::list(ListId::DrivesMore)
        );
        assert_eq!(
            PageLink::of_ida(crate::gfx::typefaces::fonts::TITLE.ida() as u32)
                .font()
                .unwrap(),
            crate::gfx::typefaces::fonts::TITLE
        );
        assert!(PageLink::of_ida(PALETTE_IDA).is_blended_palette());
        match PageLink::of_ida(TYPED_IDA + 2).place().unwrap() {
            crate::place::BytePlace::Store(record, at) => {
                assert!(std::ptr::addr_eq(
                    record,
                    &raw const crate::install::owner::install().paths
                ));
                assert_eq!(at, core::mem::offset_of!(InstallerPaths, typed) + 2);
            }
            _ => panic!("the typed folder is a place in the paths record"),
        }
    }

    #[test]
    fn a_link_is_resolved_only_as_its_own_kind() {
        assert!(PageLink::from_raw(0).in_pages().is_err());
        assert!(
            PageLink::of_ida(crate::gfx::typefaces::fonts::TITLE.ida() as u32)
                .in_pages()
                .is_err()
        );
        assert!(PageLink::of_ida(EDIT_IDA).font().is_err());
        assert!(PageLink::of_ida(TABLE_IDA + 4 * 11).box_offset().is_err());
        assert!(
            PageLink::from_raw(TABLE << KIND_SHIFT | TABLE_LEN)
                .in_pages()
                .is_err()
        );
        assert!(
            PageLink::from_raw(BOX << KIND_SHIFT | BOXES)
                .in_pages()
                .is_err()
        );
        assert!(
            PageLink::from_raw(LIST << KIND_SHIFT | 14)
                .in_pages()
                .is_err()
        );
        assert!(PageLink::of_ida(EDIT_IDA).edit_record().is_ok());
        assert!(
            PageLink::of_ida(list_ida(ListId::OpenInstalled))
                .in_pages()
                .is_ok()
        );
    }
}
