//! `marks::seed_noise` (0x4C1697): `init_marks`' step 4, the three noise tables from
//! `g_marksSeed`.
//!
//! In:  nothing
//! Out: eax, ebx, ecx the last entry's three values, esi 0
//!
//! See `marks/setup.rs`. One caller, `marks::init_marks`. Self-test: the original and the
//! Rust version on the game's variables with seeds of the test's, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::setup::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { seed_noise(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
