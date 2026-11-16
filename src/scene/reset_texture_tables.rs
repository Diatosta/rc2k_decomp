//! `scene::reset_texture_tables`: forgets which textures the kinds need and where they are:
//! `TexturePlaces.kind_slot_bytes` = 0; -1 throughout the kinds' two tables of textures needed
//! (`ObjectTables.video_levels`, `ObjectTables.system_levels`) and their first slots and first
//! textures, 0x400 dwords each. Four callers.

// ---- game data ----

pub fn reset_texture_tables(scene: &crate::scene::owner::Scene) {
    scene.textures.places.get().kind_slot_bytes.set(0); // TexturePlaces.kind_slot_bytes, cleared first
    // The two tables of textures needed, the first slots and the first textures: 0x1000 bytes each.
    for table in [
        &scene.objects.tables.get().video_levels,
        &scene.objects.tables.get().system_levels,
        &scene.textures.kinds.get().kind_first_slots,
        &scene.textures.kinds.get().kind_first_textures,
    ] {
        for cell in table {
            cell.set(u32::MAX);
        }
    }
}
