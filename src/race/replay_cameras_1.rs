//! `race::replay_cameras_1` (0x445BB2): each checkpoint's nearest replay camera of the set
//! "replaycam1" (faces of kind 0x3C).
//!
//! In:  nothing
//! Out: see `race::best_line::replay_cameras`
//!
//! See `race/best_line.rs`. Callers: `sub_445772` ("replaycam1") and `sub_459BCA`. Refuses
//! more checkpoints than the lists hold, no faces or 0x100000 or more. Self-test: faces and
//! checkpoints of the test's, the game's put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const SET: Cameras = CAMERAS[0];

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        replay_cameras(r, &SET)
    }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
