//! `scene::load_changed_texture_picture`: a system texture gets the picture it should have, if
//! it has another.
//!
//! Nothing if the texture wants no picture (`TexturePlaces.wanted_pictures[texture]` is -1 or 0) or
//! has the one it wants (`TexturePlaces.loaded_pictures[texture]`). Else
//! `scene::load_texture_picture` and the picture is noted as the one it has. Refuses a texture past
//! the tables.
//!
//! The rule is in the core crate: `rc2k_core::scene::pictures::changed_picture`.

use crate::errors::{BadInput, Refused};
use crate::gfx::load_picture_to_texture::upload_picture;
use crate::scene::load_texture_picture::load_texture_picture;

// ---- game data ----

pub const TEXTURES: u32 = 0x400;

/// The picture wanted for `texture`, loaded and uploaded when it is not the one in place.
// Logic: `rc2k_core::scene::pictures::changed_picture`
pub fn load_changed_texture_picture(
    race: &'static crate::race::owner::Race,
    gfx: &crate::gfx::owner::Gfx,
    texture: u32,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    if texture >= TEXTURES {
        return Err(BadInput("a texture past the pictures' tables").into());
    }
    let at = texture as usize;
    let Some(wanted) = rc2k_core::scene::pictures::changed_picture(
        scene.texture_places().wanted_pictures[at],
        || scene.texture_places().loaded_pictures[at],
    ) else {
        return Ok(());
    };
    let loaded = load_texture_picture(race, gfx, wanted, texture, scene, system)?;
    upload_picture(&loaded, gfx)?;
    scene.texture_places_mut().loaded_pictures[at] = wanted;
    Ok(())
}
