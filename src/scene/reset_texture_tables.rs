//! `scene::reset_texture_tables`: forgets which textures the kinds need and where they are:
//! `TexturePlaces.kind_slot_bytes` = 0; -1 throughout the kinds' two tables of textures needed
//! (`ObjectTables.video_levels`, `ObjectTables.system_levels`) and their first slots and first
//! textures, 0x400 dwords each.

// ---- game data ----

pub fn reset_texture_tables(scene: &crate::scene::owner::Scene) {
    scene.texture_places_mut().kind_slot_bytes = 0; // TexturePlaces.kind_slot_bytes, cleared first
    // The two tables of textures needed, the first slots and the first textures: 0x1000 bytes each.
    {
        let mut levels = scene.texture_levels_mut();
        levels.video_levels.fill(u32::MAX);
        levels.system_levels.fill(u32::MAX);
    }
    scene
        .textures
        .kinds
        .borrow_mut()
        .kind_first_slots
        .fill(u32::MAX);
    scene.textures.first_textures_mut().fill(u32::MAX);
}
