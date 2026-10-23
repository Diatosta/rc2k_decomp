//! `race::previous_camera` (0x41E654): see `race/edit_camera.rs`: unless the block shown
//! is the last, the camera edited (`dword_96E480`) of the block shown one back, the
//! skipped cameras passed over, the change spread over the next blocks that had the old
//! one (`race::spread_camera`).
//!
//! In:  nothing
//! Out: the block the last: eax the block. Else as `spread_camera` leaves them (eax, ecx),
//!      ebx the camera byte, dl the new camera, dh the old; no flags read
//!
//! Callers: `race::viewer_action` (actions 0x1E, 0x24). Refused, before anything is
//! written: every camera skipped (the original goes round for ever) and what
//! `spread_camera` refuses. Shadow mode compares. Self-test: `edit_camera::test_step`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::edit_camera::step_camera;
use crate::race::viewer::regs_entry;

fn rust(r: &Regs) -> Result<Regs, Refused> {
    unsafe { step_camera(r, -1) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
