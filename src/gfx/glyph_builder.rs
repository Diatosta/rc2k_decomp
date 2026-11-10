//! The sprite and font builder's work on one sheet: converted 2026-10-20 to
//! `rc2k_core::gfx::glyphs` (finding, coding and tabling a sheet's glyphs, in the original's
//! order). Here the game's side: the sheet's pixels through the row table (`g_rowTable`,
//! offsets in the decoded buffer now).

use crate::gfx::fill_row_table::row_address;

pub use rc2k_core::gfx::glyphs::{
    DIRECT_COLOUR, Glyph, ONE_FILE, RUN_LENGTHS, Sheet, SheetPixels, WIDE_HEAD, WITH_ALPHA, append,
    backdrop_of, direct_pixel_size, table_of,
};

/// A sheet's pixels where the row table leads, as the original reads them: the decoded
/// buffer's bytes (`gfx::picture`), the row table's offsets in them. A read or write past
/// them ends the game, as the original's would have gone wrong.
pub struct RowTable<'a>(&'a mut [u8]);

impl<'a> RowTable<'a> {
    pub fn new(pixels: &'a mut [u8]) -> Self {
        Self(pixels)
    }

    fn at(&self, x: u32, y: u32, len: usize) -> usize {
        let gfx = crate::gfx::owner::gfx();
        let at = row_address(y, gfx).wrapping_add(x) as usize;
        if at.checked_add(len).is_none_or(|end| end > self.0.len()) {
            crate::fatal::raise::end(
                "gfx::glyph_builder",
                crate::errors::BadInput("a sheet's pixel past the decoded picture"),
            )
        }
        at
    }
}

impl SheetPixels for RowTable<'_> {
    fn byte(&self, x: u32, y: u32) -> u8 {
        self.0[self.at(x, y, 1)]
    }
    fn set_byte(&mut self, x: u32, y: u32, value: u8) {
        let at = self.at(x, y, 1);
        self.0[at] = value;
    }
    fn dword(&self, x: u32, y: u32) -> u32 {
        let at = self.at(x.wrapping_mul(4), y, 4);
        u32::from_le_bytes(self.0[at..at + 4].try_into().expect("four bytes"))
    }
    fn set_dword(&mut self, x: u32, y: u32, value: u32) {
        let at = self.at(x.wrapping_mul(4), y, 4);
        self.0[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
}

/// 0x412232 on the palette at `palette` (an RVA): see `rc2k_core::gfx::glyphs::transparent_of`.
pub fn transparent_of(flags: u32, backdrop: u32, palette: &[crate::session::Dword; 256]) -> u32 {
    rc2k_core::gfx::glyphs::transparent_of(
        flags,
        backdrop,
        &std::array::from_fn(|i| palette[i].get()),
    )
}
