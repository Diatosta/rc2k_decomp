//! `race::lay_skidmarks` (0x4588DE): each race car's skidding wheels, their places turned
//! by the car's matrix, given skid points (`race::add_skid_point`) or their slots cleared;
//! see `race/skid_laying.rs`.
//!
//! In:  nothing
//! Out: eax the last car's bit, ecx the count (the rest kept)
//!
//! Refuses (before anything) a car without its block or more than eight. Caller:
//! `race::lay_skidmarks_if_on`. Shadow mode runs the original alone. Self-test: a copy whose point calls go to a stand-in noting the
//! place (the turns real), against the Rust version with the same: the game's cars with
//! their wheels' skids, grips and switches varied; all put back (skipped without a race).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::skid_laying::{lay_skidmarks, skid_point_part};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { lay_skidmarks(r, &mut skid_point_part) }?;
    Ok(())
}
