//! `scene::depth_buffer_on` (0x44E41F): the depth buffer used and written, compared with
//! less-or-equal (Direct3D render states 7 = 1, 14 = 1, 23 = 4).
//!
//! In:  nothing
//! Out: eax 4, ebx 0x17; the rest as `scene::set_render_state` leaves them (kept)
//!
//! Three calls of `scene::set_render_state`. 22 callers (the marks, the race's drawing).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::scene::set_render_state::set_render_state;

/// `scene::depth_buffer_on` (`on`) or `scene::depth_buffer_off`: states 7 and 14 1 or 0,
/// then 23 4.
pub unsafe fn depth_buffer(r: &mut Regs, on: bool) {
    unsafe {
        set_render_state(r, on as u32, 7);
        set_render_state(r, on as u32, 0xE);
        set_render_state(r, 4, 0x17);
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { depth_buffer(r, true) };
    Ok(())
}
