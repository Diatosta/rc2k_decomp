//! `marks::make_noise_curve` (0x4C2247): `init_marks`' step 5, the noise curve walked,
//! smoothed 0xA0 times and ramped, the random tables, the 0x20 records, the quads.
//!
//! In:  nothing
//! Out: eax, ebx, ecx, edx the last values written, esi 0, edi past the records
//!
//! See `marks/setup.rs`. One caller, `marks::init_marks`. Refuses a remainder's divisor of
//! 0, 1 or -1 (the original's `idiv` traps). Self-test: the original and the Rust version
//! on the game's tables with running sums of the test's, put back afterwards.
//!
//! Converted 2026-10-21: the walk, the smoothing, the random tables and the records through
//! `rc2k_core::marks::noise`; the roundings (`fistp`, the spread's floats, the ramp) stay
//! the x87's, as its control word says. Compared exactly as before.
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
    Ok(left_of(&unsafe { make_noise_curve(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
