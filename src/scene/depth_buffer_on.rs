//! `scene::depth_buffer_on`: the depth buffer used and written, compared with less-or-equal
//! (Direct3D render states 7 = 1, 14 = 1, 23 = 4).
//!
//! Three calls of `scene::set_render_state`. 22 callers (the marks, the race's drawing).

use crate::scene::set_render_state::set_render_state;

/// `scene::depth_buffer_on` (`on`) or `scene::depth_buffer_off`: states 7 and 14 1 or 0,
/// then 23 4.
pub fn depth_buffer(part: &crate::scene::workers::PartScratch, on: bool) {
    set_render_state(part, on as u32, 7);
    set_render_state(part, on as u32, 0xE);
    set_render_state(part, 4, 0x17);
}
