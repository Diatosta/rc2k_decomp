//! `scene::raise_texture_needs` (0x423F97): the textures of one tag are needed at a
//! level at least.
//!
//! Takes: the level, the tag
//!
//! For every texture whose tag (`dword_5CDAE0`) is ebx: its level in each of the two tables
//! (`dword_5CEAE0`, `dword_5CFAE0`) becomes eax if eax is above it, unsigned (so -1, "not
//! needed", is never raised). 57 callers.

// ---- Rust version ----

/// Whether any entry changed.
pub fn raise_texture_needs(level: u32, tag: u32, scene: &crate::scene::owner::Scene) -> bool {
    let objects = &scene.objects;
    // Converted: `rc2k_core::scene::textures::Needs::raise`.
    crate::scene::textures::with_cells(
        [
            &objects.tables.texture_tags,
            &objects.tables.system_levels,
            &objects.tables.video_levels,
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
