//! `scene::depth_test_off`: Direct3D render state 14 (writing to the depth buffer) 0.
//!
//! One call of `scene::set_render_state`. 14 callers.

use crate::scene::set_render_state::set_render_state;

pub fn depth_test_off(part: &crate::scene::workers::PartScratch) {
    set_render_state(part, 0, 0xE);
}
