//! `d3d::set_texture`: makes a texture the one polygons are drawn with. Takes the texture's number,
//! or 0xFFFFFC43 for none. Nothing if it is the current one. The number is kept in the part's
//! render cache (`d3d::render_cache`) and nothing more.

/// The texture drawing part `part`'s polygons are drawn with from now on.
pub fn set_texture(part: &crate::scene::workers::PartScratch, texture: u32) {
    crate::d3d::render_cache::set_texture(part, texture);
}
