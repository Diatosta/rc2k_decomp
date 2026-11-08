//! `scene::depth_buffer_on` (0x44E41F): the depth buffer used and written, compared with
//! less-or-equal (Direct3D render states 7 = 1, 14 = 1, 23 = 4).
//!
//! Three calls of `scene::set_render_state`. 22 callers (the marks, the race's drawing). The
//! registers above are the original's; a call by address goes through `routes.rs`, which maps
//! them.

use crate::scene::set_render_state::set_render_state;

/// `scene::depth_buffer_on` (`on`) or `scene::depth_buffer_off`: states 7 and 14 1 or 0,
/// then 23 4. What they leave in eax and ebx is read by none of the original's callers
/// (register liveness, 2026-10-23; one `depth_buffer_on` in `race::draw_screen_quad`
/// reaches the loading bar's callback, which its caller runs).
pub fn depth_buffer(on: bool) {
    set_render_state(on as u32, 7);
    set_render_state(on as u32, 0xE);
    set_render_state(4, 0x17);
}
