//! `gfx::free_fonts` (0x415C55): gives back the memory of the fonts' and sprites' glyph
//! tables.
//!
//! In:  nothing
//! Out: every register preserved
//! `memory::free_list` on `g_glyphTablesMemory` (0x51B984), and `g_fontPalettes`
//! (0x51DEB0) = 0, which is what lets `gfx::build_fonts` build them again.
//! Five callers; `sub_47486C` calls it after the main menu is left.
//! Refuses what `free_list` refuses.

use crate::errors::BadInput;
use crate::gfx::owner::gfx;
use crate::memory::free_list::free_list_at;

// ---- Rust version ----

pub fn free_fonts(
    list: crate::pointers::GamePtr,
    palettes: &crate::session::Dword,
) -> Result<(), BadInput> {
    free_list_at(list)?;
    palettes.set(0);
    Ok(())
}

/// With the game's list and palettes' place.
pub fn free_fonts_in_game() -> Result<(), BadInput> {
    let fonts = &gfx().fonts;
    let list = crate::pointers::GamePtr::to(&fonts.glyph_tables.memory);
    free_fonts(list, &fonts.menu_fonts.palettes)
}
