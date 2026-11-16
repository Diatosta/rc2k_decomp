//! `scene::clear_slot_textures`: no texture slot holds or wants a texture.
//!
//! -1 throughout the slots' loaded and wanted textures, 0x400 dwords each, so that every slot is
//! loaded anew.

// ---- game data ----

pub fn clear_slot_textures(scene: &crate::scene::owner::Scene) {
    let textures = &scene.textures;
    // The slots' loaded and wanted textures: 0x1000 bytes each, 0xFF.
    for table in [
        &textures.places.get().slot_loaded,
        &textures.places.get().slot_wanted,
    ] {
        for cell in table {
            cell.set(u32::MAX);
        }
    }
}
