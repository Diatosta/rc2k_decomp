//! `scene::set_texture` (0x44E77E): `d3d::set_texture` with the texture, unless the draw switch is 0.
//!
//! Takes: the texture's number
//!
//! 21 callers, none reading flags. `d3d::set_texture` is the platform's.

/// `d3d::set_texture` with the texture, unless the switch is 0.
use crate::scene::owner::scene;
pub fn set_texture(texture: u32) {
    if scene().objects.tables.d3d_drawing_on.get() != 0 {
        crate::d3d::set_texture::set_texture(texture);
    }
}
