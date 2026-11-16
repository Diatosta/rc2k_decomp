//! Gives back the fonts' glyph tables (`memory::free_list` on their list).

use crate::errors::Refused;
pub fn free_glyph_tables(gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    { crate::gfx::glyph_store::free_list(&gfx.fonts, &gfx.fonts.glyph_tables.memory) }?;
    Ok(())
}
