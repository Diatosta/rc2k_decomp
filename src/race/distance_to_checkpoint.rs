//! `race::distance_to_checkpoint` (0x44141D): how far checkpoint ecx's face's first point
//! is in x and z from (`dword_639824`, `dword_63982C`), into `dword_62400C`; -1 for a
//! checkpoint outside 0 .. `g_checkpointCount` - 1 (sign tests).
//!
//! In:  ecx = the checkpoint
//! Out: `dword_62400C`; with a checkpoint eax = the z difference, ebx and esi = the point's
//!      number and address; the rest preserved
//!
//! See `race/start_place.rs`. One caller, `race::nearest_checkpoint_in` (0x4413F8). The
//! length: `fild` / `fmul st, st` twice, `faddp`, `fsqrt`, `fistp` (`x87::int_hypot`).
//! Its first bytes hold a `js`: private copy (the whole function, no calls), manual
//! patch. Refuses a checkpoint inside the count but past the list (0x1418). Shadow mode: the
//! Rust version, the variable put back, the original's, compared. Self-test: faces,
//! points and checkpoints of the test's (`TestLine`), the game's put back.
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
    let g = unsafe { distance_to_checkpoint(r) }?;
    Ok(left_changed(r, &g))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
