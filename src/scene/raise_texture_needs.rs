//! `scene::raise_texture_needs`: the textures of one tag are needed at a level at least.
//!
//! For every texture whose tag is the given tag: its level in each of the two tables (system
//! and video) becomes the given level if that is above it, unsigned (so -1, "not needed", is
//! never raised). 57 callers.

/// Whether any entry changed.
pub fn raise_texture_needs(level: u32, tag: u32, scene: &crate::scene::owner::Scene) -> bool {
    let objects = &scene.objects;
    // Logic: `rc2k_core::scene::textures::Needs::raise`.
    crate::scene::textures::with_cells(
        [
            &objects.tables.get().texture_tags,
            &objects.tables.get().system_levels,
            &objects.tables.get().video_levels,
        ],
        |[tags, system, video]| {
            rc2k_core::scene::textures::Needs {
                tags,
                system,
                video,
            }
            .raise(level, tag)
        },
    )
}
