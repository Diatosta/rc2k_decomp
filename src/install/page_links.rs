//! The links in the installer pages' data, as handles: an item names its box, its edit
//! record, its value or its text, a drawing list's record its font or palette. Each dword holds a handle, made once where the data is built
//! (`vars::InstallerPages::new`) and resolved by its kind, with bounds, by every reader. A
//! handle is a kind in the high byte and a number below:
//!
//! - `BOX`: one of `InstallerPages::boxes`, by index (0 the title item's, 1 the title's);
//! - `EDIT`: the folder item's edit record;
//! - `FONT`: a font of the font block, by number (`gfx::typefaces::FontId`);
//! - `BLENDED_PALETTE`: the blended palette (the install bar's glyphs);
//! - `TYPED_PATH`: a byte offset in `InstallerPaths::typed`.
//!
//! 0 is no link. A kind resolves only as itself: a box link is no text, a font link no box.

use crate::errors::BadInput;
use crate::gfx::typefaces::FontId;
use crate::install::list_pages::{BoxRect, EditRecord};
use crate::install::owner::Install;
use crate::memory::place::BytePlace;

const KIND_SHIFT: u32 = 24;
const NUMBER: u32 = (1 << KIND_SHIFT) - 1;
const FONT: u32 = 2;
const BLENDED_PALETTE: u32 = 3;
const TYPED_PATH: u32 = 4;
const BOX: u32 = 6;
const EDIT: u32 = 8;
/// The bytes of the folder typed a link can name.
const TYPED_LEN: u32 = 0x100;

/// How many `InstallerPages::boxes` there are (16 bytes each).
const BOXES: u32 = 9;

/// One of the installer pages' lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListId {
    Language,
    Main,
    /// The drive page's list for up to 8 drives, up to 16, and more.
    Drives8,
    Drives16,
    DrivesMore,
    Folder,
    Room,
    Removal,
}

impl ListId {
    /// The lists, in the order of the enum.
    pub(crate) const ALL: [ListId; 8] = [
        ListId::Language,
        ListId::Main,
        ListId::Drives8,
        ListId::Drives16,
        ListId::DrivesMore,
        ListId::Folder,
        ListId::Room,
        ListId::Removal,
    ];
}

/// A link of the installer pages' data (a handle; see the module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageLink(u32);

impl PageLink {
    pub const fn folder_edit() -> Self {
        Self(EDIT << KIND_SHIFT)
    }

    pub const fn of_font(font: FontId) -> Self {
        Self((FONT << KIND_SHIFT) | font.number())
    }

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

    /// The box a link of the `BOX` kind names.
    pub fn box_rect(self, install: &'static Install) -> Result<&'static BoxRect, BadInput> {
        match (self.kind(), self.number()) {
            (BOX, number) if number < BOXES => Ok(&install.pages.boxes[number as usize]),
            _ => Err(BadInput("a page link that is no box")),
        }
    }

    pub fn edit_record(self, install: &'static Install) -> Result<&'static EditRecord, BadInput> {
        match self.kind() {
            EDIT if self.number() == 0 => Ok(&install.pages.folder_edit),
            _ => Err(BadInput("a page link that is no edit record")),
        }
    }

    /// The place a text or a byte link names: the folder typed (bounded by the installer's path
    /// buffers); the pages hold no other text or byte an item sets.
    pub fn place(self, install: &'static Install) -> Result<BytePlace, BadInput> {
        if self.kind() == TYPED_PATH && self.number() < TYPED_LEN {
            return Ok(BytePlace::In(&install.paths.typed, self.number() as usize));
        }
        Err(BadInput(
            "a page link that is no place in the installer's pages",
        ))
    }

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
/// the link's raw dword is its value.
#[repr(transparent)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct LinkCell(crate::memory::session::PlainDword);

impl LinkCell {
    pub const fn new(link: PageLink) -> Self {
        Self(crate::memory::session::PlainDword::new(link.raw()))
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
        let install = crate::test_support::own::install();
        let pages = &crate::test_support::own::install().pages;
        assert!(core::ptr::eq(
            PageLink::box_at(1).box_rect(install).unwrap(),
            &raw const pages.boxes[1]
        ));
        assert_eq!(
            PageLink::of_font(crate::gfx::typefaces::fonts::TITLE)
                .font()
                .unwrap(),
            crate::gfx::typefaces::fonts::TITLE
        );
        assert!(PageLink::blended_palette().is_blended_palette());
        match PageLink::typed(2).place(install).unwrap() {
            crate::memory::place::BytePlace::In(buffer, at) => {
                assert!(std::ptr::addr_eq(
                    buffer,
                    &raw const crate::test_support::own::install().paths.typed
                ));
                assert_eq!(at, 2);
            }
            _ => panic!("the typed folder is a place in the paths record's buffer"),
        }
    }

    #[test]
    fn a_link_is_resolved_only_as_its_own_kind() {
        let install = crate::test_support::own::install();
        assert!(PageLink::from_raw(0).box_rect(install).is_err());
        assert!(PageLink::from_raw(0).place(install).is_err());
        assert!(
            PageLink::of_font(crate::gfx::typefaces::fonts::TITLE)
                .box_rect(install)
                .is_err()
        );
        assert!(PageLink::folder_edit().font().is_err());
        assert!(PageLink::box_at(3).place(install).is_err());
        assert!(
            PageLink::from_raw(BOX << KIND_SHIFT | BOXES)
                .box_rect(install)
                .is_err()
        );
        assert!(
            PageLink::from_raw(TYPED_PATH << KIND_SHIFT | TYPED_LEN)
                .place(install)
                .is_err()
        );
        assert!(PageLink::folder_edit().edit_record(install).is_ok());
    }
}
