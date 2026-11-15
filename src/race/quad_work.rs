//! The screen quad's inputs, a view's own working values: the colour, the texture coordinates and
//! the corners (`quad_corners`: [0] left, [1] right, [4] top, [5] bottom), each view's own. One
//! per view slot (`race::view_scratch::ViewScratch::quad`).

use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells)]
pub struct QuadWork {
    /// The quad's colour and see-through (`scene::draw_flat_quad`, `scene::draw_tinted_quad`).
    pub colour: [PlainDword; 2],
    /// The quad's texture coordinates (`race::frame_flush`, `hud::draw_view_strip`).
    pub uv: [PlainDword; 8],
    /// The quad's corners.
    pub corners: [PlainDword; 8],
}

impl QuadWork {
    pub const fn new() -> Self {
        Self {
            colour: [const { PlainDword::new(0) }; 2],
            uv: [const { PlainDword::new(0) }; 8],
            corners: [const { PlainDword::new(0) }; 8],
        }
    }
}
