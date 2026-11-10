//! `gfx::free_glyph_tables` (0x474D12): `memory::free_list` on `g_glyphTablesMemory`
//! (0x51B984): the fonts' glyph tables given back.
//!
//! Caller: `menu::apply_screen_mode` (between `gfx::free_fonts` and `gfx::build_fonts`).
//! Called directly.

use crate::errors::Refused;
pub fn free_glyph_tables(gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    { crate::memory::free_list::free_list(&gfx.fonts.glyph_tables.memory) }?;
    Ok(())
}
