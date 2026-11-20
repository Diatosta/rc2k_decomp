//! `scene::look_up_texture`: the polygon's texture for the texture word `SceneDraw.texture_page`:
//! -1 none; with the top bit a place (`TexturePlaces.place_slot`, below 0xFA0); else a kind's first
//! slot (below 0x400); a slot not below the texture slot count (or none loaded) none (0xFFFFFC43).

/// Looks up the polygon's texture.
pub fn look_up_texture(
    poly: &crate::scene::polygon_work_store::PolygonWork,
    scene: &crate::scene::owner::Scene,
) -> u32 {
    // Logic: `rc2k_core::scene::textures::look_up`.
    let found = rc2k_core::scene::textures::look_up(
        poly.texture_page.get(),
        |p| scene.textures.places.get().place_slot[p].get(),
        |k| scene.textures.kinds.get().kind_first_slots[k].get(),
        || scene.textures.places.get().totals[2].get(),
    );
    poly.polygon_texture.set(found.texture);
    found.last_number
}
