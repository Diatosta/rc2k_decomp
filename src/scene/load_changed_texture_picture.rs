//! `scene::load_changed_texture_picture` (0x42574B): a system texture gets the picture
//! it should have, if it has another.
//!
//! In:  ecx = the texture
//! Out: every register preserved
//!
//! Nothing if the texture wants no picture (`dword_5D8968[texture]` is -1 or 0) or has the one
//! it wants (`dword_5D9968[texture]`). Else `scene::load_texture_picture` and the picture is
//! noted as the one it has. One caller, `scene::reload_system_textures`. Refuses a texture past
//! the tables.
//!
//! Converted 2026-10-21: the rule through `rc2k_core::scene::pictures::changed_picture`.

use crate::errors::{BadInput, Refused};
use crate::scene::load_texture_picture::load_texture_picture_in_game;
use crate::scene::texture_places_store::TEXTURE_PLACES;

// ---- game data ----

pub const TEXTURES: u32 = 0x400;

// ---- Rust version ----

/// `load`: what stands for `scene::load_texture_picture`, given the picture and the
/// texture.
// Converted: `rc2k_core::scene::pictures::changed_picture`
pub unsafe fn load_changed_texture_picture(
    texture: u32,
    load: &mut dyn FnMut(u32, u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    if texture >= TEXTURES {
        return Err(BadInput("a texture past the pictures' tables").into());
    }
    let at = texture as usize;
    let Some(wanted) = rc2k_core::scene::pictures::changed_picture(
        TEXTURE_PLACES.wanted_pictures[at].get(),
        || TEXTURE_PLACES.loaded_pictures[at].get(),
    ) else {
        return Ok(());
    };
    load(wanted, texture)?;
    TEXTURE_PLACES.loaded_pictures[at].set(wanted);
    Ok(())
}

pub unsafe fn load_changed_texture_picture_in_game(texture: u32) -> Result<(), Refused> {
    unsafe {
        load_changed_texture_picture(texture, &mut |picture, texture| {
            load_texture_picture_in_game(picture, texture)
        })
    }
}
