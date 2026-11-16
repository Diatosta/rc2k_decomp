//! `scene::reset_texture_places`: no texture has a place: -1 throughout
//! `TexturePlaces.loaded_pictures` and `TexturePlaces.wanted_pictures` (each texture's place),
//! 0x400 dwords each.

// ---- game data ----

pub fn reset_texture_places(scene: &crate::scene::owner::Scene) {
    let textures = &scene.textures;
    // `loaded_pictures`, `wanted_pictures`: 0x1000 bytes each, 0xFF.
    for table in [
        &textures.places.get().loaded_pictures,
        &textures.places.get().wanted_pictures,
    ] {
        for cell in table {
            cell.set(u32::MAX);
        }
    }
}
