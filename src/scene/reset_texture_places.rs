//! `scene::reset_texture_places`: no texture has a place: -1 throughout
//! `TexturePlaces.loaded_pictures` and `TexturePlaces.wanted_pictures` (each texture's place),
//! 0x400 dwords each.

// ---- game data ----

pub fn reset_texture_places(scene: &crate::scene::owner::Scene) {
    // `loaded_pictures`, `wanted_pictures`: 0x1000 bytes each, 0xFF.
    let mut places = scene.texture_places_mut();
    places.loaded_pictures.fill(u32::MAX);
    places.wanted_pictures.fill(u32::MAX);
}
