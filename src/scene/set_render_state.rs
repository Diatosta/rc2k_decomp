//! `scene::set_render_state` (0x44EB72): one of Direct3D's render states.
//!
//! Takes: the value, the state
//!
//! A call of `d3d::set_render_state` and nothing more. 208 call sites. The registers above are
//! the original's; a call by address goes through `routes.rs`, which maps them.

/// `d3d::set_render_state` with the value and the state. Its leftovers are read by none
/// of the original's 198 callers (register liveness, 2026-10-23).
pub fn set_render_state(value: u32, state: u32) {
    crate::d3d::set_render_state::set_render_state(value, state);
}
