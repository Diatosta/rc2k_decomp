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
use crate::gfx::block::game_addr;
use crate::gfx::build_fonts::PALETTES_RVA;
use crate::gfx::glyph_tables::TABLES_LIST_RVA;
use crate::memory::MemoryList;
use crate::memory::free_list::free_list;

// ---- Rust version ----

pub unsafe fn free_fonts(list: *mut MemoryList, palettes: *mut u32) -> Result<(), BadInput> {
    unsafe {
        free_list(list)?;
        palettes.write_unaligned(0);
        Ok(())
    }
}

/// With the game's list and palettes' place.
pub fn free_fonts_in_game() -> Result<(), BadInput> {
    let list = game_addr(TABLES_LIST_RVA) as *mut MemoryList;
    let palettes = game_addr(PALETTES_RVA) as *mut u32;
    unsafe { free_fonts(list, palettes) }
}
