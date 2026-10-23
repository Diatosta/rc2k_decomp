//! `race::edit_camera_2` (0x41E5BA): see `race/edit_camera.rs`: the replay kept for an
//! undo (`race::keep_undo`) and, unless the block shown is the last, `dword_96E480` 2.
//!
//! In:  nothing
//! Out: eax the block shown (`dword_5C2530`); the rest kept
//!
//! Callers: `race::viewer_action` (actions 0x19, 0x1D; no flags read). Shadow mode
//! compares. Self-test: `edit_camera::test_edit`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::edit_camera::edit_camera;
use crate::race::viewer::regs_entry;

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(edit_camera(r, 2))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
