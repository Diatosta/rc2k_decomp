//! `scene::depth_test_on` (0x44E3E1): Direct3D render state 14 (writing to the depth buffer)
//! 1.
//!
//! In:  nothing
//! Out: eax 1, ebx 0xE; the rest as `scene::set_render_state` leaves them (kept)
//!
//! One call of `scene::set_render_state`. 15 callers. Self-test: the original and the Rust
//! version recorded on the layer, nothing drawn.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::scene::set_render_state::set_render_state;

/// The Rust version (see the header).
pub unsafe fn depth_test_on(r: &mut Regs) {
    unsafe { set_render_state(r, 1, 0xE) };
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { depth_test_on(r) };
    Ok(())
}
