//! Gives back the memory of the fonts' and sprites' glyph tables: `memory::free_list` on the glyph
//! tables' list, and the font palettes' place = 0, which is what lets `gfx::build_fonts` build them
//! again. `menu::run_front_end` calls it after the main menu is left. Refuses what `free_list`
//! refuses.

use crate::errors::BadInput;
use crate::gfx::glyph_store::free_list;
use crate::gfx::owner::FontStores;

pub fn free_fonts(fonts: &FontStores) -> Result<(), BadInput> {
    free_list(fonts, &fonts.glyph_tables.memory)?;
    fonts.font_block.palettes.set(0);
    Ok(())
}

/// With the game's list and palettes' place.
pub fn free_fonts_in_game(gfx: &crate::gfx::owner::Gfx) -> Result<(), BadInput> {
    free_fonts(&gfx.fonts)
}
