//! `marks::find_spline_ends` (0x4BA4F1): record ebp's highest point of grade 0 and lowest
//! of grade 1.
//!
//! In:  ebp the spline record
//! Out: eax the second count, ecx the second found, edx its value, edi the last point read
//!      times 12
//!
//! See `marks/splines.rs`. Callers: `sub_4B9A32` (two). Reads its values 0x30 a point apart
//! (section 16), as the original. Refuses a negative start or count, or points past 0x80.
//! Self-test: the original and the Rust version on a record of the test's (with room for
//! those reads), the refusal.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { find_spline_ends(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
