//! `scene::begin_scene` (0x44E8A1): the 3D part of a frame begins.
//!
//! In:  nothing
//! Out: the registers and flags `d3d::begin_scene` leaves (carry = it failed)
//!
//! `d3d::begin_scene` (0x41ABC7: Direct3D's BeginScene and the layer's own states,
//! platform), then, whatever it says, `scene::set_filters` and render states 0x15 = 4
//! (the texture's blending), 0x1C = 1 (fog on), 0x13 = 5 and 0x14 = 6 (the source and
//! destination blend), 0x1B = 1 (blending on), all between `pusha` / `pushf` and
//! `popf` / `popa`. Nine callers, `menu::frame_callback` among them.
//! Hook removed 2026-10-22; plain Rust since 2026-10-23 (phase 3): the registers above
//! are the original's; a call by address goes through `routes.rs`, which maps them.

use crate::scene::set_filters::set_filters;
use crate::scene::set_render_state::set_render_state;

/// The states after the filters: (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (1, 0x1C), (5, 0x13), (6, 0x14), (1, 0x1B)];

/// The 3D part of a frame begun. The layer's beginning cannot fail: what the original
/// leaves is eax 0, the carry clear and the zero flag set, which register-threaded
/// callers set themselves.
pub fn begin_scene() {
    crate::platform_3d::begin_layer();
    set_filters();
    for (value, state) in STATES {
        set_render_state(value, state);
    }
}
