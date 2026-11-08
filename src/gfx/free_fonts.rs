//! `gfx::free_fonts` (0x415C55): gives back the memory of the fonts' and sprites' glyph
//! tables.
//!
//! `memory::free_list` on `g_glyphTablesMemory` (0x51B984), and `g_fontPalettes`
//! (0x51DEB0) = 0, which is what lets `gfx::build_fonts` build them again.
//! Five callers; `menu::run_front_end` calls it after the main menu is left.
//! Refuses what `free_list` refuses.

use crate::errors::BadInput;
use crate::memory::free_list::free_list;

// ---- Rust version ----

pub fn free_fonts(
    list: &dyn crate::pointers::Store,
    palettes: &crate::session::Dword,
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
