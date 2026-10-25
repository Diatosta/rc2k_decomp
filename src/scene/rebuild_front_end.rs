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
//! `set_up_front_end_scene`. Refuses what its calls refuse.

use crate::errors::Refused;
use crate::gfx::fonts_store::FONTS;
use crate::gfx::texture_pixels::GRADING;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::session::Dword;

const SPRITE_TABLE: &Dword = &LOOP_FLAGS.sprite_table; // g_spriteTable

/// The Rust version (the original keeps every register: `pusha` / `popa`).
pub fn rebuild_front_end() -> Result<(), Refused> {
    unsafe {
        if FONTS.palettes.get() == 0 {
            crate::gfx::build_fonts::build_fonts()?;
            crate::gfx::build_menu_sprites::build_menu_sprites(SPRITE_TABLE.addr() as *mut u32)?;
        }
        crate::scene::set_up_lit_engine::set_up_lit_engine()?;
        crate::scene::set_up_engine::set_up_engine()?;
        GRADING.reading.set(0);
        GRADING.scale.set(0x100);
        GRADING.saturation.set(0x100);
        crate::scene::set_up_front_end_scene::set_up_front_end_scene()?;
        crate::scene::set_up_engine::set_up_engine()
    }
}
