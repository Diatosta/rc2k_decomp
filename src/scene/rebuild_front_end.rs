//! `scene::rebuild_front_end` (0x4154CD): the front end made again: without fonts
//! (`g_fontPalettes` 0) `gfx::build_fonts` and `gfx::build_menu_sprites`; then
//! `scene::set_up_lit_engine`, `scene::set_up_engine`, `dword_4F4A24` 0, `dword_4F4A28`
//! and `dword_4F4A2C` 0x100, `scene::set_up_front_end_scene`, `scene::set_up_engine`;
//! all inside `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_418DF4` (the main loop's other way, not ported). Its calls by place: 0
//! `build_fonts`, 1 `build_menu_sprites`, 2 `set_up_lit_engine`, 3 `set_up_engine`, 4
//! `set_up_front_end_scene`.
//! Refuses what its calls refuse. Shadow mode runs the original alone (it loads and sets
//! up). Self-test: a copy whose calls go to stand-ins that note them, against the Rust
//! version with the same, fonts there or not (the game's variables put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, rva};
use crate::gfx::picture_store::Slot as PicSlot;

const FONT_PALETTES: usize = 0x51DEB0; // g_fontPalettes
const SPRITE_TABLE: usize = 0x755581; // g_spriteTable
/// `dword_4F4A24`, `dword_4F4A28`, `dword_4F4A2C` and their values.
const SETTINGS: [(PicSlot, u32); 3] = [
    (PicSlot::at(0x4F4A24), 0),
    (PicSlot::at(0x4F4A28), 0x100),
    (PicSlot::at(0x4F4A2C), 0x100),
];

/// The Rust version (the original keeps every register: `pusha` / `popa`).
pub fn rebuild_front_end() -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(rva(FONT_PALETTES)) == 0 {
            crate::gfx::build_fonts::build_fonts()?;
            crate::gfx::build_menu_sprites::build_menu_sprites(
                game_addr(rva(SPRITE_TABLE)) as *mut u32
            )?;
        }
        crate::scene::set_up_lit_engine::set_up_lit_engine()?;
        crate::scene::set_up_engine::set_up_engine()?;
        for (at, v) in SETTINGS {
            crate::gfx::block::game_write(at, v);
        }
        crate::scene::set_up_front_end_scene::set_up_front_end_scene()?;
        crate::scene::set_up_engine::set_up_engine()
    }
}
