//! Gives back the memory of the fonts' and sprites' glyph tables: `memory::free_list` on the glyph
//! tables' list, and the font palettes' place = 0, which is what lets `gfx::build_fonts` build them
//! again. `menu::run_front_end` calls it after the main menu is left. Refuses what `free_list`
//! refuses.

use crate::errors::BadInput;
use crate::memory::free_list::free_list;

pub fn free_fonts(
    list: &crate::memory::ListCells,
    palettes: &crate::session::PlainDword,
) -> Result<(), BadInput> {
    free_list(list)?;
    palettes.set(0);
    Ok(())
}

/// With the game's list and palettes' place.
pub fn free_fonts_in_game(gfx: &crate::gfx::owner::Gfx) -> Result<(), BadInput> {
    let fonts = &gfx.fonts;
    let list = &fonts.glyph_tables.memory;
    free_fonts(list, &fonts.font_block.palettes)
}
