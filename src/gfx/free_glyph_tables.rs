//! Gives back the fonts' glyph tables (`memory::free_list` on their list).

use crate::errors::Refused;
pub fn free_glyph_tables(gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    { crate::memory::free_list::free_list(&gfx.fonts.glyph_tables.memory) }?;
    Ok(())
}
