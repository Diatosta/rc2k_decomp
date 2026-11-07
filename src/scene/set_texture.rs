//! `scene::set_texture` (0x44E77E): `d3d::set_texture` with eax, unless `dword_5CDA8C` is 0.
//!
//! Takes: the texture's number
//!
//! 21 callers, none reading flags. `d3d::set_texture` is the platform's.

/// `d3d::set_texture` with the texture, unless the switch is 0. What it leaves in the
/// registers is read by none of its callers (register liveness, 2026-10-23).
use crate::scene::owner::scene;
pub fn set_texture(texture: u32) {
    if scene().objects.tables.has_3d.get() != 0 {
        crate::d3d::set_texture::set_texture(texture);
    }
}
