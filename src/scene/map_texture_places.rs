//! `scene::map_texture_places`: for each texture's place, the slot that shows it.
//!
//! The 0xFA0 dwords of `TexturePlaces.place_slot` = -1; then for the slots from 0x3FF down to 0:
//! the texture the slot wants (`TexturePlaces.slot_wanted`), if under 0x400, has a place
//! (`TexturePlaces.wanted_pictures`), and that place, if under 0xFA0, gets the slot's number. Of
//! two slots with one place the lower stays.

pub fn map_texture_places(scene: &crate::scene::owner::Scene) {
    // Logic: `rc2k_core::scene::textures::map_places`.
    let places = scene.texture_places();
    let mut slots = scene.textures.place_slots.borrow_mut();
    rc2k_core::scene::textures::map_places(
        &places.slot_wanted,
        &places.wanted_pictures,
        &mut slots.place_slot,
    );
}
