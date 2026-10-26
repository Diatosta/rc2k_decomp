//! `drive::finish_race_0` (0x4278C7): `dword_621870` 0 and `drive::finish_race`.
//!
//! In:  nothing
//! Out: as `finish_race` leaves the registers; no flags read
//!
//! Callers: `sub_488155`, `sub_4A14CB`, `sub_4A9439` (the championship's flow). Ungated.
//! Refuses what `finish_race` refuses. Shadow mode runs the original alone.
//! Self-test: skipped (it loads a stage and plays the finish routine).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::p;
use crate::screens::harness::Left;

/// The Rust version.
pub unsafe fn finish_race_0(entry: &Regs) -> Result<Left, Refused> {
    p(0x621870, 0);
    unsafe { crate::drive::finish_race::finish_race(entry) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { finish_race_0(r) }?;
    left.apply(r);
    Ok(())
}
