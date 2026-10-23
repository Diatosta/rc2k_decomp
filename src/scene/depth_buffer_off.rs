//! `scene::depth_buffer_off` (0x44E3F1): the depth buffer neither used nor written
//! (Direct3D render states 7 = 0, 14 = 0, 23 = 4).
//!
//! In:  nothing
//! Out: eax 4, ebx 0x17; the rest as `scene::set_render_state` leaves them (kept)
//!
//! See `scene/depth_buffer_on.rs`. 11 callers. Self-test: recorded on the layer, nothing
//! drawn.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::scene::depth_buffer_on::depth_buffer;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { depth_buffer(r, false) };
    Ok(())
}
