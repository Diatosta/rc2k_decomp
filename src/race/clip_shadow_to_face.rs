//! `race::clip_shadow_to_face` (0x457960): the shadow's polygon cut to one course face
//! (see `race/car_shadow.rs`: `sub_457A85` per edge, `sub_457BE8` the side test and
//! `sub_457CA8` the crossing, covered here; the x87 the original's instructions).
//!
//! In:  nothing (the face's corners `dword_641938` ..)
//! Out: the registers as it leaves them (its caller calls it inside `pusha`)
//!
//! Shadow mode: by state. Self-test: scene points of the test's (`g_scenePoints` pointed
//! at them), a square cut by triangles inside, across and away, the work car near and far;
//! all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::car_shadow::*;
use crate::race::frame_draw::all_regs;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { clip_shadow_to_face(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
