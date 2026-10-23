//! `scene::load_changed_texture_picture` (0x42574B): a system texture gets the picture
//! it should have, if it has another.
//!
//! In:  ecx = the texture
//! Out: every register preserved
//!
//! Nothing if the texture wants no picture (`dword_5D8968[texture]` is -1 or 0) or
//! has the one it wants (`dword_5D9968[texture]`). Else `scene::load_texture_picture`
//! and the picture is noted as the one it has.
//! One caller, `scene::reload_system_textures`.
//! Refuses a texture past the tables.
//! Shadow mode runs the original alone and checks what the texture has afterwards.
//! Self-test: a redirected copy whose loading notes its registers.
//!
//! Converted 2026-10-21: the rule through `rc2k_core::scene::pictures::changed_picture`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::scene::load_texture_picture::load_texture_picture_in_game;

// ---- game data ----

pub const WANTED_PICTURES_RVA: usize = rva(0x5D8968); // dword_5D8968
pub const LOADED_PICTURES_RVA: usize = rva(0x5D9968); // dword_5D9968
pub const TEXTURES: u32 = 0x400;

// ---- Rust version ----

/// `load`: what stands for `scene::load_texture_picture`, given the picture and the
/// texture.
// Converted: `rc2k_core::scene::pictures::changed_picture`
pub unsafe fn load_changed_texture_picture(
    texture: u32,
    load: &mut dyn FnMut(u32, u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        if texture >= TEXTURES {
            return Err(BadInput("a texture past the pictures' tables").into());
        }
        let at = texture as usize * 4;
        let Some(wanted) = rc2k_core::scene::pictures::changed_picture(
            game_read(WANTED_PICTURES_RVA + at),
            || game_read(LOADED_PICTURES_RVA + at),
        ) else {
            return Ok(());
        };
        load(wanted, texture)?;
        game_write(LOADED_PICTURES_RVA + at, wanted);
        Ok(())
    }
}

pub unsafe fn load_changed_texture_picture_in_game(texture: u32) -> Result<(), Refused> {
    unsafe {
        load_changed_texture_picture(texture, &mut |picture, texture| {
            load_texture_picture_in_game(picture, texture)
        })
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { load_changed_texture_picture_in_game(r.ecx) }?;
    Ok(())
}
