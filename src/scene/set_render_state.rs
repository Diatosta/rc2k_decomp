//! `scene::set_render_state` (0x44EB72): one of Direct3D's render states.
//!
//! In:  eax = the value, ebx = the state
//! Out: as `d3d::set_render_state` leaves them: every register preserved
//!
//! A call of `d3d::set_render_state` and nothing more. 208 call sites.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, rva};

const LAYER_RVA: usize = rva(0x41AB5B); // d3d::set_render_state

// ---- Rust version ----

/// `d3d::set_render_state` with the value and the state, the other registers as given.
pub unsafe fn set_render_state(r: &mut Regs, value: u32, state: u32) {
    r.eax = value;
    r.ebx = state;
    crate::calls::call(LAYER_RVA, r);
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let (value, state) = (r.eax, r.ebx);
    unsafe { set_render_state(r, value, state) };
    Ok(())
}
