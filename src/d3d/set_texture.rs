//! `d3d::set_texture` (0x41ACF5): makes a texture the one polygons are drawn with.
//!
//! Takes: the texture's number, or 0xFFFFFC43 for none
//! Nothing if it is the current one (`g_currentTexture`, 0x5AB93C); else that is set
//! and the device told: render state 1 with the texture's handle from the texture
//! table (0x78 bytes a texture) and a light state, or render state 1
//! with 0 for none. A second entry point five bytes before it loads the "none" texture and runs into it. Four callers.
//! Platform: the original always runs, here it is only noted. In the standalone
//! program, and for a recording that swallows, the number is kept and nothing more.

/// The texture polygons are drawn with from now on (the standalone keeps its number).
pub fn set_texture(texture: u32) {
    crate::d3d::render_cache::set_texture(texture);
}
