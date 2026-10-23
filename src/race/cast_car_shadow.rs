//! `race::cast_car_shadow` (0x4575C1): the work car's shadow (see `race/car_shadow.rs`):
//! with three corners or more its points, order, hull and polygon, the hull's list, and
//! the shadow on the faces round the car (`race::shadow_faces`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller calls it inside `pusha`)
//!
//! Refuses what its parts refuse. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose faces call goes to a stand-in (the rest real), against the
//! Rust version with the same; the work car's corners set; all put back (skipped without
//! a race).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::car_shadow::*;

/// The shadow on the faces, in the game.
pub fn faces_in_game(r: &Regs) -> Result<Regs, crate::errors::Refused> {
    unsafe { shadow_faces(r, &mut |e| shadow_on_face(e, &mut face_part).map(|_| ())) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { cast_car_shadow(r, &mut faces_in_game) }?;
    Ok(())
}
