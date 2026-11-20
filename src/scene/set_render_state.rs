//! `scene::set_render_state`: one of Direct3D's render states: a call of
//! `d3d::set_render_state` with the value and the state.

/// `d3d::set_render_state` with the value and the state, in drawing part `part`'s cache.
pub fn set_render_state(part: &crate::scene::scratch_store::PartScratch, value: u32, state: u32) {
    crate::d3d::set_render_state::set_render_state(part, value, state);
}
