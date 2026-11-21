//! `scene::assign_menu_textures`: the eight slots of kind 0x258 get the menu's own textures.
//!
//! For each of the eight numbers of `Carousel::textures` that is not -1: the slot (kind 0x258's
//! first slot, `KindSlots.kind_first_slots[600]`, plus its place in the eight) wants that
//! number plus kind 0x258's first texture (`Textures::first_textures[600]`). Refuses a
//! slot past the table.

use crate::errors::BadInput;
use crate::menu::carousel_store::Carousel;

// ---- game data ----
const KIND: usize = 0x258;

pub fn assign_menu_textures(
    carousel: &Carousel,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    let textures = &scene.textures;
    // Logic: `rc2k_core::scene::textures::menu_textures`.
    let wanted = rc2k_core::scene::textures::menu_textures(
        carousel.read().textures,
        textures.kinds.borrow().kind_first_slots[KIND],
        textures.first_textures()[KIND],
    )?;
    let mut places = scene.texture_places_mut();
    for (slot, texture) in wanted {
        *places
            .slot_wanted
            .get_mut(slot as usize)
            .ok_or(BadInput("a slot past the texture places' slots"))? = texture;
    }
    Ok(())
}
