//! `scene::depth_test_on` (0x44E3E1): Direct3D render state 14 (writing to the depth buffer)
//! 1.
//!
//! In:  nothing
//! Out: eax 1, ebx 0xE; the rest as `scene::set_render_state` leaves them (kept)
//!
//! One call of `scene::set_render_state`. 15 callers.

use crate::scene::set_render_state::set_render_state;

/// The Rust version (see the header).
pub fn depth_test_on() {
    set_render_state(1, 0xE);
}
