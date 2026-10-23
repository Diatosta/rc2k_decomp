//! `race::nearest_checkpoint_in` (0x4413F8): of checkpoints ecx up to edx (once at least:
//! the test is after the step, `jb`), the nearest by `race::distance_to_checkpoint`.
//!
//! In:  ecx = the first, edx = the end
//! Out: ebp = the nearest (the later of equals, distances unsigned; 0 if none), edi = its
//!      distance (-1 if none), ecx = past the last; eax, ebx, esi as the last measure
//!      leaves them; the rest preserved; `dword_62400C` the last distance
//!
//! See `race/start_place.rs`. One caller, `race::nearest_checkpoint` (0 .. count).
//! Refuses more than 0x1418 measures or checkpoints, before anything. Shadow mode: the
//! Rust version, the variable put back, the original's, compared. Self-test: faces,
//! points and checkpoints of the test's (`TestLine`), the game's put back; ranges inside,
//! across and past the list, an empty one.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::start_place::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { nearest_checkpoint_in(r) }?;
    Ok(left_changed(r, &g))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
