//! The links in the installer pages' data, as handles.
//!
//! The original's pages hold addresses: a page entry names its list, an item its box, its
//! choices, its edit record, its value or its text, a list its neighbours table, a drawing
//! list's record its font or palette. The data here holds a handle in each of those dwords,
//! made once where the data is built (`vars::linked`, from IDA's address), and every reader
//! resolves it with bounds. A handle is a kind in the high byte and a number below:
//!
//! - `PAGES`: a byte offset in `InstallerPages` (the table, list, item, box, text or byte);
//! - `FONT`: a font of the font block, by number (`gfx::typefaces::FontId`);
//! - `BLENDED_PALETTE`: the blended palette (the install bar's glyphs);
//! - `TYPED_PATH`: a byte offset in `InstallerPaths::typed` (the folder item's text starts
//!   past the drive's `c:`).
//!
//! 0 is no link (a page with no next entry, the title's box).

use crate::errors::BadInput;
use crate::gfx::typefaces::{self, FontId};
use crate::install::vars::{InstallerPages, InstallerPaths};
use crate::pointers::GamePtr;

const KIND_SHIFT: u32 = 24;
const NUMBER: u32 = (1 << KIND_SHIFT) - 1;
const PAGES: u32 = 1;
const FONT: u32 = 2;
const BLENDED_PALETTE: u32 = 3;
const TYPED_PATH: u32 = 4;

/// IDA's address of `InstallerPaths::typed` (`byte_95EBF8`): the folder typed on the page. The
/// folder item's text starts two bytes in (0x95EBFA, past the drive's `c:`), or at its start
/// on Unix (no drive).
const TYPED_IDA: u32 = 0x95EBF8;
/// IDA's address of the blended palette the install bar's glyphs name.
const PALETTE_IDA: u32 = 0x4F3B74;
/// The bytes `InstallerPages` holds before IDA's first (`vars::InstallerPages::home`).
const PAGES_LEAD: u32 = 7;

/// A link of the installer pages' data (a handle; see the module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageLink(u32);

impl PageLink {
    /// The link the original's address `ida` is (where the data is built; an address that
    /// names none of the kinds fails to compile).
    pub const fn of_ida(ida: u32) -> Self {
        let pages = InstallerPages::START as u32;
        if ida >= pages && ida < pages + InstallerPages::LEN as u32 {
            return Self((PAGES << KIND_SHIFT) | (ida - pages + PAGES_LEAD));
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

    /// The link to byte `offset` of `InstallerPages` (a field's `offset_of!`).
    pub const fn in_pages_at(offset: usize) -> Self {
        assert!(offset < size_of::<InstallerPages>());
        Self((PAGES << KIND_SHIFT) | offset as u32)
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

    /// The place in the installer pages the link names, bounded by the pages (the table or
    /// text may run on inside them as the original's does).
    pub fn in_pages(self) -> Result<GamePtr, BadInput> {
        let install = crate::install::owner::install();
        if self.kind() != PAGES || self.number() as usize >= size_of::<InstallerPages>() {
            return Err(BadInput(
                "a page link that is no place in the installer's pages",
            ));
        }
        Ok(GamePtr::to(&install.pages).offset(self.number()))
    }

    /// The place a text or a byte link names: in the pages, or the folder typed (bounded by
    /// the installer's path buffers).
    pub fn place(self) -> Result<GamePtr, BadInput> {
        let install = crate::install::owner::install();
        if self.kind() == TYPED_PATH && self.number() < 0x100 {
            let typed = core::mem::offset_of!(InstallerPaths, typed) as u32;
            return Ok(GamePtr::to(&install.paths).offset(typed + self.number()));
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

    #[test]
    fn the_original_addresses_become_the_handles_of_their_kind() {
        let title = PageLink::of_ida(0x51DD78);
        assert_eq!(
            title,
            PageLink::in_pages_at(core::mem::offset_of!(InstallerPages, title))
        );
        assert_eq!(
            PageLink::of_ida(0x51E8A4).font().unwrap(),
            FontId::at(0x51E8A4)
        );
        assert!(PageLink::of_ida(PALETTE_IDA).is_blended_palette());
        assert_eq!(
            Some(GamePtr::to(&crate::text::blend_font::BLENDED_PALETTE).addr() as usize),
            crate::moved::home_of_rva(PALETTE_IDA as usize - crate::framework::IMAGE_BASE)
        );
        let typed = PageLink::of_ida(TYPED_IDA + 2).place().unwrap();
        assert_eq!(
            typed.addr(),
            GamePtr::to(&crate::install::owner::install().paths.typed)
                .addr()
                .wrapping_add(2)
        );
    }

    #[test]
    fn a_link_is_resolved_only_as_its_own_kind() {
        assert!(PageLink::from_raw(0).in_pages().is_err());
        assert!(PageLink::of_ida(0x51E8A4).in_pages().is_err());
        assert!(PageLink::of_ida(0x51DD78).font().is_err());
        assert!(
            PageLink::from_raw(PAGES << KIND_SHIFT | 0xFFFF)
                .in_pages()
                .is_err()
        );
        let last = size_of::<InstallerPages>() as u32 - 1;
        assert!(
            PageLink::from_raw(PAGES << KIND_SHIFT | last)
                .in_pages()
                .is_ok()
        );
    }
}
