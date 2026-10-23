//! `race::car_part_matrices` (0x460E7C): car esi's marks' switches and its four wheels'
//! matrices (see `race/car_parts.rs`).
//!
//! In:  esi = the car's block
//! Out: the registers as it leaves them (inside its caller's `pusha`)
//!
//! Refuses (before anything) axle values whose scaling would fault. Caller:
//! `race::ready_car_parts`. Shadow mode runs the original alone. Self-test: a copy (its
//! calls to the game's, hooked) against the Rust version on a copy of car 0's block, its
//! state, switches, broken parts and size varied; everything put back (skipped without a
//! race).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::car_parts::car_part_matrices;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { car_part_matrices(r.esi) }?;
    Ok(())
}
