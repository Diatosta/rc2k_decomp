//! `race::prepare_sound_tables` (0x40D636): once (`dword_516358`) the 0x22 sound records
//! (`off_514F90`) made ready (spans, the reciprocal at +0xC, three rows each) and the 0x40
//! floats at 0x516258 1 + 0.05 sin(x), x from 0 by pi / 32.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/sound_tables.rs`. One caller, `race::set_up_race_sound_objects` (0x415C93).
//! Shadow mode: the Rust version, the records and tables put back, the original's,
//! compared (the floats within `FLOAT`: the sines go through `fsin`, which no precision
//! rounds). The Rust version is `prepare_sound_tables_plain`. Self-test: a copy whose 20
//! references to the four variables are the test's (records of its own), against the
//! Rust version on the same: not done (records filled two ways) and done.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::sound_tables::*;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { prepare_sound_tables_plain(&TableVars::game()) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
