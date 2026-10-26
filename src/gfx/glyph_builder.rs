//! The sprite and font builder's work on one sheet: converted 2026-10-20 to
//! `rc2k_core::gfx::glyphs` (finding, coding and tabling a sheet's glyphs, in the original's
//! order). Here the game's side: the sheet's pixels through the row table (`g_rowTable`).

use crate::gfx::fill_row_table::row_address;

pub use rc2k_core::gfx::glyphs::{
    DIRECT_COLOUR, Glyph, ONE_FILE, RUN_LENGTHS, Sheet, SheetPixels, WIDE_HEAD, WITH_ALPHA, append,
    backdrop_of, direct_pixel_size, table_of,
};

/// A sheet's pixels where the row table leads, as the original reads them (checked, through
/// a window on the sheet's block).
#[derive(Default)]
pub struct RowTable(crate::pointers::Window);

impl RowTable {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SheetPixels for RowTable {
    fn byte(&self, x: u32, y: u32) -> u8 {
        self.0
            .read_or_end(row_address(y).wrapping_add(x), "a sheet's pixel")
    }
    fn set_byte(&mut self, x: u32, y: u32, value: u8) {
        self.0
            .write_or_end(row_address(y).wrapping_add(x), value, "a sheet's pixel")
    }
    fn dword(&self, x: u32, y: u32) -> u32 {
        self.0.read_or_end(
            row_address(y).wrapping_add(x.wrapping_mul(4)),
            "a sheet's pixel",
        )
    }
    fn set_dword(&mut self, x: u32, y: u32, value: u32) {
        self.0.write_or_end(
            row_address(y).wrapping_add(x.wrapping_mul(4)),
            value,
            "a sheet's pixel",
        )
    }
}

/// `sub_412232` on the palette at `palette` (an RVA): see `rc2k_core::gfx::glyphs::transparent_of`.
pub fn transparent_of(flags: u32, backdrop: u32, palette: &[crate::session::Dword; 256]) -> u32 {
    rc2k_core::gfx::glyphs::transparent_of(
        flags,
        backdrop,
        &std::array::from_fn(|i| palette[i].get()),
    )
}
