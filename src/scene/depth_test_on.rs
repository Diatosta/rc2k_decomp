//! `scene::depth_test_on` (0x44E3E1): Direct3D render state 14 (writing to the depth buffer)
//! 1.
//!
//! One call of `scene::set_render_state`. 15 callers.

use crate::scene::set_render_state::set_render_state;

pub fn depth_test_on(part: &crate::workers::PartScratch) {
    set_render_state(part, 1, 0xE);
}
