//! `scene::set_render_state` (0x44EB72): one of Direct3D's render states.
//!
//! Takes: the value, the state
//!
//! A call of `d3d::set_render_state` and nothing more.

/// `d3d::set_render_state` with the value and the state, in drawing part `part`'s cache.
pub fn set_render_state(part: &crate::workers::PartScratch, value: u32, state: u32) {
    crate::d3d::set_render_state::set_render_state(part, value, state);
}
