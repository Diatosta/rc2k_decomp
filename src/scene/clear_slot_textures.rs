//! `scene::clear_slot_textures`: no texture slot holds or wants a texture.
//!
//! -1 throughout the slots' loaded and wanted textures, 0x400 dwords each, so that every slot is
//! loaded anew.

// ---- game data ----

pub fn clear_slot_textures(scene: &crate::scene::owner::Scene) {
    // The slots' loaded and wanted textures: 0x1000 bytes each, 0xFF.
    let mut places = scene.texture_places_mut();
    places.slot_loaded.fill(u32::MAX);
    places.slot_wanted.fill(u32::MAX);
}
