//! `race::checkpoint_distances` (0x445AF1): the distance along the best line at each
//! checkpoint, and the whole.
//!
//! In:  ecx (only with fewer than two checkpoints: where the 0 goes)
//! Out: edi the length, ecx where it was put; with two checkpoints or more, eax, ebx, esi
//!      the last point's z, number and address; the rest preserved
//!
//! See `race/best_line.rs`. Callers: `sub_445772` ("dists") and `sub_459BCA`. Refuses a
//! last checkpoint of 0 or past the lists, and with fewer than two checkpoints an ecx past
//! the list. Self-test: checkpoints of the test's, the game's put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        checkpoint_distances(r)
    }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
