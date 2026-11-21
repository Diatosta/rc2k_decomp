//! `scene::set_texture`: `d3d::set_texture` with the texture, unless the draw switch is 0. 21
//! callers.

use crate::scene::owner::Scene;

/// `d3d::set_texture` with the texture (drawing part `part`'s), unless the switch is 0: on the
/// game's thread, outside the drawing parts.
pub fn set_texture(part: &crate::scene::scratch_store::PartScratch, scene: &Scene, texture: u32) {
    set_texture_with(
        part,
        crate::scene::object_tables_store::ObjectKindsRef::Store(&scene.objects.tables),
        texture,
    );
}

/// `set_texture` on the object tables as the caller reads them (a drawing part's from its
/// `scene::workers::DrawReads`).
pub fn set_texture_with(
    part: &crate::scene::scratch_store::PartScratch,
    tables: crate::scene::object_tables_store::ObjectKindsRef<'_>,
    texture: u32,
) {
    if tables.d3d_drawing_on() != 0 {
        crate::d3d::set_texture::set_texture(part, texture);
    }
}
