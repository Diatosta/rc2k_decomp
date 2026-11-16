//! `scene::map_texture_places`: for each texture's place, the slot that shows it.
//!
//! The 0xFA0 dwords of `TexturePlaces.place_slot` = -1; then for the slots from 0x3FF down to 0:
//! the texture the slot wants (`TexturePlaces.slot_wanted`), if under 0x400, has a place
//! (`TexturePlaces.wanted_pictures`), and that place, if under 0xFA0, gets the slot's number. Of
//! two slots with one place the lower stays.

use crate::scene::textures::with_cells;

pub fn map_texture_places(scene: &crate::scene::owner::Scene) {
    let textures = &scene.textures;
    // Logic: `rc2k_core::scene::textures::map_places`.
    with_cells(
        [
            &textures.places.get().slot_wanted,
            &textures.places.get().wanted_pictures,
            &textures.places.get().place_slot,
        ],
        |[wanted, pictures, places]| {
            rc2k_core::scene::textures::map_places(wanted, pictures, places)
        },
    );
}
