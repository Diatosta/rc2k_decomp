//! `gfx::free_fonts` (0x415C55): gives back the memory of the fonts' and sprites' glyph
//! tables.
//!
//! In:  nothing
//! Out: every register preserved
//! `memory::free_list` on `g_glyphTablesMemory` (0x51B984), and `g_fontPalettes`
//! (0x51DEB0) = 0, which is what lets `gfx::build_fonts` build them again.
//! Five callers; `sub_47486C` calls it after the main menu is left.
//! Shadow mode runs the original alone (the memory is given back once) and checks what
//! it left. Self-test: a redirected copy with a list and a variable of the test's.
//! Refuses what `free_list` refuses.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::{Regs, game_addr};
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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    let list = game_addr(TABLES_LIST_RVA) as *mut MemoryList;
    let palettes = game_addr(PALETTES_RVA) as *mut u32;
    if let Err(e) = unsafe { free_fonts(list, palettes) } {
        return Err(e.into());
    }
    Ok(())
}
