//! The links in the installer pages' data, as handles: a drawing list's record names its font or
//! palette, the page's font table its fonts. Each dword holds a handle, made once where the data is
//! built (`vars::InstallerPages::new`) and resolved by its kind, with bounds, by every reader. A
//! handle is a kind in the high byte and a number below:
//!
//! - `FONT`: a font of the font block, by number (`gfx::typefaces::FontId`);
//! - `BLENDED_PALETTE`: the blended palette (the install bar's glyphs).
//!
//! 0 is no link. A kind resolves only as itself: a font link is no palette.

use crate::errors::BadInput;
use crate::gfx::typefaces::FontId;

const KIND_SHIFT: u32 = 24;
const NUMBER: u32 = (1 << KIND_SHIFT) - 1;
const FONT: u32 = 2;
const BLENDED_PALETTE: u32 = 3;

/// A link of the installer pages' data (a handle; see the module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageLink(u32);

impl PageLink {
    pub(super) const fn of_font(font: FontId) -> Self {
        Self((FONT << KIND_SHIFT) | font.number())
    }

    pub const fn blended_palette() -> Self {
        Self(BLENDED_PALETTE << KIND_SHIFT)
    }

    /// A link as the pages' data holds it.
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// The pages' dword for this link.
    pub const fn raw(self) -> u32 {
        self.0
    }

    fn kind(self) -> u32 {
        self.0 >> KIND_SHIFT
    }

    fn number(self) -> u32 {
        self.0 & NUMBER
    }

    pub fn font(self) -> Result<FontId, BadInput> {
        match FontId::try_numbered(self.number() as usize) {
            Some(font) if self.kind() == FONT => Ok(font),
            _ => Err(BadInput("a page link that is no font")),
        }
    }

    /// Whether the link names the blended palette (the only palette the pages name).
    pub(super) fn is_blended_palette(self) -> bool {
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
        assert_eq!(
            PageLink::of_font(crate::gfx::typefaces::fonts::TITLE)
                .font()
                .unwrap(),
            crate::gfx::typefaces::fonts::TITLE
        );
        assert!(PageLink::blended_palette().is_blended_palette());
        assert!(PageLink::blended_palette().font().is_err());
        assert!(PageLink::from_raw(0).font().is_err());
        assert!(!PageLink::of_font(crate::gfx::typefaces::fonts::TITLE).is_blended_palette());
    }
}
