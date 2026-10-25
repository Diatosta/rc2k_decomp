//! The sprite and font builder's work on one sheet: converted 2026-10-20 to
//! `rc2k_core::gfx::glyphs` (finding, coding and tabling a sheet's glyphs, in the original's
//! order). Here the game's side: the sheet's pixels through the row table (`g_rowTable`).

use crate::gfx::fill_row_table::row_address;

pub use rc2k_core::gfx::glyphs::{
    DIRECT_COLOUR, Glyph, ONE_FILE, RUN_LENGTHS, Sheet, SheetPixels, WIDE_HEAD, WITH_ALPHA, append,
    backdrop_of, direct_pixel_size, table_of,
};

/// A sheet's pixels where the row table leads, as the original reads them.
pub struct RowTable;

impl SheetPixels for RowTable {
    fn byte(&self, x: u32, y: u32) -> u8 {
        unsafe { (row_address(y).wrapping_add(x) as *const u8).read() }
    }
    fn set_byte(&mut self, x: u32, y: u32, value: u8) {
        unsafe { (row_address(y).wrapping_add(x) as *mut u8).write(value) }
    }
    fn dword(&self, x: u32, y: u32) -> u32 {
        unsafe { (row_address(y).wrapping_add(x.wrapping_mul(4)) as *const u32).read_unaligned() }
    }
    fn set_dword(&mut self, x: u32, y: u32, value: u32) {
        unsafe {
            (row_address(y).wrapping_add(x.wrapping_mul(4)) as *mut u32).write_unaligned(value)
        }
    }
}

/// `sub_412232` on the palette at `palette` (an RVA): see `rc2k_core::gfx::glyphs::transparent_of`.
pub unsafe fn transparent_of(
    flags: u32,
    backdrop: u32,
    palette: &[crate::session::Dword; 256],
) -> u32 {
    rc2k_core::gfx::glyphs::transparent_of(
        flags,
        backdrop,
        &std::array::from_fn(|i| palette[i].get()),
    )
}
