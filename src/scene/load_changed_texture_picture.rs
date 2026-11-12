//! `scene::load_changed_texture_picture` (0x42574B): a system texture gets the picture
//! it should have, if it has another.
//!
//! Takes: the texture
//!
//! Nothing if the texture wants no picture (`TexturePlaces.wanted_pictures[0][texture]` is -1 or 0) or has the one
//! it wants (`TexturePlaces.loaded_pictures[0][texture]`). Else `scene::load_texture_picture` and the picture is
//! noted as the one it has. One caller, `scene::reload_system_textures`. Refuses a texture past
//! the tables.
//!
//! The logic is in the core crate: the rule through `rc2k_core::scene::pictures::changed_picture`.

use crate::errors::{BadInput, Refused};
use crate::scene::load_texture_picture::load_texture_picture_in_game;

// ---- game data ----

pub const TEXTURES: u32 = 0x400;

// ---- Rust version ----

/// `load`: what stands for `scene::load_texture_picture`, given the picture and the
/// texture.
// Converted: `rc2k_core::scene::pictures::changed_picture`
pub fn load_changed_texture_picture(
    texture: u32,
    load: &mut dyn FnMut(u32, u32) -> Result<(), Refused>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let textures = &scene.textures;
    if texture >= TEXTURES {
        return Err(BadInput("a texture past the pictures' tables").into());
    }
    let at = texture as usize;
    let Some(wanted) = rc2k_core::scene::pictures::changed_picture(
        textures.places.wanted_pictures[at].get(),
        || textures.places.loaded_pictures[at].get(),
    ) else {
        return Ok(());
    };
    load(wanted, texture)?;
    textures.places.loaded_pictures[at].set(wanted);
    Ok(())
}

pub fn load_changed_texture_picture_in_game(
    race: &'static crate::race::owner::Race,
    gfx: &crate::gfx::owner::Gfx,
    texture: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    load_changed_texture_picture(
        texture,
        &mut |picture, texture| load_texture_picture_in_game(race, gfx, picture, texture, scene),
        scene,
    )
}
