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
use crate::gfx::typefaces::FontId;
use crate::install::pages_map::PagesAt;
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
/// The bytes of the folder typed a link can name.
const TYPED_LEN: u32 = 0x100;

/// How many entries `InstallerPages::page_table` holds (IDA 0x51D594 ..).
pub const TABLE_LEN: u32 = 55;
/// How many `InstallerPages::boxes` there are (16 bytes each, IDA 0x51DD88 ..).
const BOXES: u32 = 9;

/// One of the installer pages' lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListId {
    Language,   // IDA 0x51D670
    Main,       // IDA 0x51D728
    Mode,       // IDA 0x51D790
    Drives8,    // IDA 0x51D7F8
    Drives16,   // IDA 0x51D8B0
    DrivesMore, // IDA 0x51D9E8
    Folder,     // IDA 0x51DBA0
    Room,       // IDA 0x51DC18
    Dxsetup,    // IDA 0x51DC60
    Finish,     // IDA 0x51DCB8
    /// The pages that are only an action (a header with no items): copying the files,
    /// uninstalling, opening the installed copy, asking to register.
    InstallFiles, // IDA 0x51DD10
    Uninstall,  // IDA 0x51DD28
    OpenInstalled, // IDA 0x51DD40
    AskRegistration, // IDA 0x51DD58
}

/// The installer's state, for the place a `PageLink` or a list handle names. A link and a
/// handle are copied numbers with no owner, so this is a seam: the install pages' functions
/// that hold a `ListItem`, `ListMenu` or `PageLink` would pass `&Install` to them.
pub fn installer() -> &'static crate::install::owner::Install {
    crate::install::owner::install() // m6-root
}

impl ListId {
    /// The lists, in the order of the enum.
    pub(crate) const ALL: [ListId; 14] = [
        ListId::Language,
        ListId::Main,
        ListId::Mode,
        ListId::Drives8,
        ListId::Drives16,
        ListId::DrivesMore,
        ListId::Folder,
        ListId::Room,
        ListId::Dxsetup,
        ListId::Finish,
        ListId::InstallFiles,
        ListId::Uninstall,
        ListId::OpenInstalled,
        ListId::AskRegistration,
    ];

    const fn number(self) -> u32 {
        self as u32
    }

    const fn from_number(number: u32) -> Option<Self> {
        if number < Self::ALL.len() as u32 {
            Some(Self::ALL[number as usize])
        } else {
            None
        }
    }
}

/// A link of the installer pages' data (a handle; see the module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageLink(u32);

impl PageLink {
    /// The link to page entry `index` of the pages' table.
    pub const fn table(index: u32) -> Self {
        assert!(index < TABLE_LEN);
        Self((TABLE << KIND_SHIFT) | index)
    }

    /// The link to one of the pages' lists.
    pub const fn list(list: ListId) -> Self {
        Self((LIST << KIND_SHIFT) | list.number())
    }

    /// The link to the folder item's edit record.
    pub const fn folder_edit() -> Self {
        Self(EDIT << KIND_SHIFT)
    }

    /// The link to a font of the font block.
    pub const fn of_font(font: FontId) -> Self {
        Self((FONT << KIND_SHIFT) | font.number())
    }

    /// The link to the blended palette (the install bar's glyphs).
    pub const fn blended_palette() -> Self {
        Self(BLENDED_PALETTE << KIND_SHIFT)
    }

    /// The link to byte `offset` of the folder typed (`InstallerPaths::typed`; the folder
    /// item's text starts two bytes in, past the drive's `c:`, or at its start on Unix).
    pub const fn typed(offset: u32) -> Self {
        assert!(offset < TYPED_LEN);
        Self((TYPED_PATH << KIND_SHIFT) | offset)
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

    /// The page table entry a link names.
    pub fn table_index(self) -> Option<u32> {
        (self.kind() == TABLE && self.number() < TABLE_LEN).then_some(self.number())
    }

    /// Where the thing of the installer pages the link names starts.
    pub fn pages_at(self) -> Option<PagesAt> {
        let number = self.number();
        match self.kind() {
            TABLE if number < TABLE_LEN => Some(PagesAt::table(number)),
            LIST => ListId::from_number(number).map(PagesAt::list),
            BOX if number < BOXES => Some(PagesAt::box_at(number)),
            EDIT if number == 0 => Some(PagesAt::edit()),
            _ => None,
        }
    }

    /// The place in the installer pages the link names (a table entry, a list, a box, a
    /// handler or the edit record), bounded by the pages (a table or text may run on inside
    /// them as the original's does).
    pub fn in_pages(self) -> Result<BytePlace, BadInput> {
        Ok(self.pages_start()?.place())
    }

    /// The position in the pages of the place the link names.
    pub fn pages_start(self) -> Result<PagesAt, BadInput> {
        self.pages_at().ok_or(BadInput(
            "a page link that is no place in the installer's pages",
        ))
    }

    /// The position in the pages of the box a link names.
    pub fn box_position(self) -> Result<PagesAt, BadInput> {
        match self.pages_at() {
            Some(at) if self.kind() == BOX => Ok(at),
            _ => Err(BadInput("a page link that is no box")),
        }
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
            EDIT if self.number() == 0 => Ok(&installer().pages.folder_edit),
            _ => Err(BadInput("a page link that is no edit record")),
        }
    }

    /// The place a text or a byte link names: in the pages, or the folder typed (bounded by
    /// the installer's path buffers).
    pub fn place(self) -> Result<BytePlace, BadInput> {
        if self.kind() == TYPED_PATH && self.number() < TYPED_LEN {
            return Ok(crate::install::pages_map::typed_folder(self.number()));
        }
        self.in_pages()
    }

    /// The font the link names.
    pub fn font(self) -> Result<FontId, BadInput> {
        match FontId::try_numbered(self.number() as usize) {
            Some(font) if self.kind() == FONT => Ok(font),
            _ => Err(BadInput("a page link that is no font")),
        }
    }

    /// Whether the link names the blended palette (the only palette the pages name).
    pub fn is_blended_palette(self) -> bool {
        self.0 == BLENDED_PALETTE << KIND_SHIFT
    }
}

/// A dword of the pages' data that holds a link (a drawing list record's font or palette):
/// the link's raw dword is its four bytes in the pages' run.
#[repr(transparent)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct LinkCell(crate::session::PlainDword);

impl LinkCell {
    pub const fn new(link: PageLink) -> Self {
        Self(crate::session::PlainDword::new(link.raw()))
    }

    pub fn get(&self) -> PageLink {
        PageLink::from_raw(self.0.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_handles_resolve_as_their_own_kind() {
        let title_box = PageLink::box_at(1);
        assert_eq!(title_box.box_position().unwrap(), PagesAt::box_at(1));
        assert_eq!(
            PageLink::table(25).table_index().unwrap(),
            25,
            "the drive page's entry"
        );
        assert_eq!(
            PageLink::of_font(crate::gfx::typefaces::fonts::TITLE)
                .font()
                .unwrap(),
            crate::gfx::typefaces::fonts::TITLE
        );
        assert!(PageLink::blended_palette().is_blended_palette());
        match PageLink::typed(2).place().unwrap() {
            crate::place::BytePlace::In(buffer, at) => {
                assert!(std::ptr::addr_eq(
                    buffer,
                    &raw const crate::install::owner::install().paths.typed
                ));
                assert_eq!(at, 2);
            }
            _ => panic!("the typed folder is a place in the paths record's buffer"),
        }
    }

    #[test]
    fn a_link_is_resolved_only_as_its_own_kind() {
        assert!(PageLink::from_raw(0).in_pages().is_err());
        assert!(
            PageLink::of_font(crate::gfx::typefaces::fonts::TITLE)
                .in_pages()
                .is_err()
        );
        assert!(PageLink::folder_edit().font().is_err());
        assert!(PageLink::table(11).box_position().is_err());
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
        assert!(PageLink::folder_edit().edit_record().is_ok());
        assert!(PageLink::list(ListId::OpenInstalled).in_pages().is_ok());
    }
}
