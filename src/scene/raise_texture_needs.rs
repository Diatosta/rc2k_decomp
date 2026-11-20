//! `scene::raise_texture_needs`: the textures of one tag are needed at a level at least.
//!
//! For every texture whose tag is the given tag: its level in each of the two tables (system
//! and video) becomes the given level if that is above it, unsigned (so -1, "not needed", is
//! never raised). 57 callers.

/// Whether any entry changed.
pub fn raise_texture_needs(level: u32, tag: u32, scene: &crate::scene::owner::Scene) -> bool {
    // Logic: `rc2k_core::scene::textures::Needs::raise`.
    let levels = &mut *scene.texture_levels_mut();
    rc2k_core::scene::textures::Needs {
        tags: &levels.texture_tags,
        system: &mut levels.system_levels,
        video: &mut levels.video_levels,
    }
    .raise(level, tag)
}
