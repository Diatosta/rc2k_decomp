//! `scene::reset_texture_tables` (0x42247B): forgets which textures the kinds need and where they are.
//!
//! `TexturePlaces.kind_slot_bytes` = 0; -1 throughout the kinds' two tables of textures needed (`ObjectTables.video_levels[0]`,
//! `ObjectTables.system_levels[0]`) and their first slots and first textures (`g_kindFirstSlots`,
//! `g_kindFirstTextures`), 0x400 dwords each. Four callers.

// ---- game data ----

// ---- Rust version ----

pub fn reset_texture_tables(scene: &crate::scene::owner::Scene) {
    scene.textures.places.kind_slot_bytes.set(0); // TexturePlaces.kind_slot_bytes, cleared first
    // ObjectTables.video_levels[0], ObjectTables.system_levels[0], g_kindFirstSlots, g_kindFirstTextures: 0x1000 bytes each.
    for table in [
        &scene.objects.tables.video_levels,
        &scene.objects.tables.system_levels,
        &scene.textures.kinds.kind_first_slots,
        &scene.textures.kinds.kind_first_textures,
    ] {
        for cell in table {
            cell.set(u32::MAX);
        }
    }
}
