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
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::race::noted_calls::Part;

pub const FONTS: u32 = 0;
pub const SPRITES: u32 = 1;
pub const LIT_ENGINE: u32 = 2;
pub const ENGINE: u32 = 3;
pub const FRONT_END: u32 = 4;
const FONT_PALETTES: usize = 0x51DEB0; // g_fontPalettes
const SPRITE_TABLE: usize = 0x755581; // g_spriteTable
/// `dword_4F4A24`, `dword_4F4A28`, `dword_4F4A2C` and their values.
const SETTINGS: [(usize, u32); 3] = [(0x4F4A24, 0), (0x4F4A28, 0x100), (0x4F4A2C, 0x100)];

/// The steps (the registers kept).
pub fn rebuild_front_end_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let mut g = *r;
    if unsafe { game_read::<u32>(rva(FONT_PALETTES)) } == 0 {
        part(FONTS, &mut g)?;
        part(SPRITES, &mut g)?;
    }
    part(LIT_ENGINE, &mut g)?;
    part(ENGINE, &mut g)?;
    for (at, v) in SETTINGS {
        unsafe { game_write(rva(at), v) };
    }
    part(FRONT_END, &mut g)?;
    part(ENGINE, &mut g)
}

/// The calls in the game: the Rust versions.
pub fn front_end_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            FONTS => crate::gfx::build_fonts::build_fonts(r, &mut crate::gfx::glyph_tables::Game)?,
            SPRITES => crate::gfx::build_menu_sprites::build_menu_sprites(
                r,
                &mut crate::gfx::glyph_tables::Game,
                game_addr(rva(SPRITE_TABLE)) as *mut u32,
            )?,
            LIT_ENGINE => crate::scene::set_up_lit_engine::set_up_lit_engine_in(
                r,
                &mut crate::scene::set_up_lit_engine::lit_part,
            )?,
            ENGINE => crate::scene::set_up_engine::set_up_engine_in_game(r),
            _ => crate::scene::set_up_front_end_scene::set_up_front_end_scene_in_game(r)?,
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let mut g = *r;
    rebuild_front_end_in(&mut g, &mut front_end_part)?;
    Ok(())
}
