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
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, rva};
use crate::scene::set_filters::set_filters;
use crate::scene::set_render_state::set_render_state;

const LAYER_RVA: usize = rva(0x41ABC7); // d3d::begin_scene
/// The states after the filters: (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (1, 0x1C), (5, 0x13), (6, 0x14), (1, 0x1B)];

// ---- Rust version ----

/// `begin` stands for `d3d::begin_scene`.
pub unsafe fn begin_scene(r: &mut Regs, begin: &mut dyn FnMut(&mut Regs)) {
    begin(r);
    set_filters();
    for (value, state) in STATES {
        set_render_state(value, state);
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { begin_scene(r, &mut |r| crate::calls::call(LAYER_RVA, r)) };
    Ok(())
}
