//! `scene::clear_slot_textures` (0x4256A8): no texture slot holds or wants a texture.
//!
//! -1 throughout `g_slotLoadedTextures` and `g_slotWantedTextures`, 0x400 dwords each, so that
//! every slot is loaded anew. One caller, `scene::reload_slot_textures`.

// ---- game data ----

// ---- Rust version ----

pub fn clear_slot_textures(scene: &crate::scene::owner::Scene) {
    let textures = &scene.textures;
    // g_slotLoadedTextures, g_slotWantedTextures: 0x1000 bytes each, 0xFF.
    for table in [
        &textures.places.get().slot_loaded,
        &textures.places.get().slot_wanted,
    ] {
        for cell in table {
            cell.set(u32::MAX);
        }
    }
}
