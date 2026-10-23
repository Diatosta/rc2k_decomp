//! `race::silence_race_sounds` (0x40D366): the race's sounds reset
//! (`race::reset_race_sounds`), command 4, the sound commands waited for.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/sound_tables.rs`. Callers: `sub_415C6D` and `sub_4430F2`. Its calls begin at
//! +1: private copy, the three calls relocated, manual patch. Shadow mode runs the
//! original alone (it sends commands). Self-test: a copy whose three calls go to
//! stand-ins, against the Rust version on the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::sound_tables::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { silence_race_sounds(r, &mut sound_part) }?;
    Ok(())
}
