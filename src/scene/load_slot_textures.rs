//! `scene::load_slot_textures`: every texture slot in use gets the texture it wants, and
//! returns the number of slots.
//!
//! `scene::load_slot_texture` for the slots 0 .. `TexturePlaces.totals[2]`. Six callers.
//! Refuses more slots than the table has.

use crate::errors::BadInput;
use crate::race::owner_frame::Frame;
use crate::scene::load_slot_texture::load_slot_texture_in_game;

/// The number of slots in use (which `scene::draw_carousel` reads in the main menu).
// Logic: `rc2k_core::scene::textures::slots_in_use`
pub fn load_slot_textures(
    load: &mut dyn FnMut(u32) -> Result<(), BadInput>,
    scene: &crate::scene::owner::Scene,
) -> Result<u32, BadInput> {
    let count =
        rc2k_core::scene::textures::slots_in_use(scene.textures.places.get().totals[2].get())?;
    for slot in 0..count {
        load(slot)?;
    }
    Ok(count)
}

/// With the game's loading.
pub fn load_slot_textures_in_game(
    frame: &Frame,
    scene: &crate::scene::owner::Scene,
) -> Result<u32, BadInput> {
    load_slot_textures(
        &mut |slot| load_slot_texture_in_game(frame, slot, scene),
        scene,
    )
}
