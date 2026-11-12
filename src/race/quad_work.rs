//! The screen quad's inputs, a view's own working values (frame plan F7): the colour, the texture
//! coordinates and the corners (`RACE_VARS.quad_colour` 0x623DE8, `RACE_VARS.quad_uv` 0x623DF0, `quad_corners` 0x623E10: [0]
//! left, [1] right, [4] top, [5] bottom), each view's own. One per view slot
//! (`race::view_scratch::ViewScratch::quad`).

use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct QuadWork {
    /// The quad's colour and see-through (`scene::draw_flat_quad`, `scene::draw_tinted_quad`).
    pub colour: [Dword; 2],
    /// The quad's texture coordinates (`race::frame_flush`, `hud::draw_view_strip`).
    pub uv: [Dword; 8],
    /// The quad's corners.
    pub corners: [Dword; 8],
}

impl QuadWork {
    pub const fn new() -> Self {
        Self {
            colour: [const { Dword::new(0) }; 2],
            uv: [const { Dword::new(0) }; 8],
            corners: [const { Dword::new(0) }; 8],
        }
    }
}
