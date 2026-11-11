//! `scene::reset_texture_places` (0x423FE1): no texture has a place.
//!
//! -1 throughout `TexturePlaces.loaded_pictures[0]` and `TexturePlaces.wanted_pictures[0]` (each texture's place), 0x400 dwords each.
//! Two callers.

// ---- game data ----

// ---- Rust version ----

pub fn reset_texture_places(scene: &crate::scene::owner::Scene) {
    let textures = &scene.textures;
    // TexturePlaces.loaded_pictures[0], TexturePlaces.wanted_pictures[0]: 0x1000 bytes each, 0xFF.
    for table in [
        &textures.places.loaded_pictures,
        &textures.places.wanted_pictures,
    ] {
        for cell in table {
            cell.set(u32::MAX);
        }
    }
}
