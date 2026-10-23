//! `physics::free_rev_in_neutral` (0x43EA54): `physics::free_rev` in neutral.
//!
//! In:  nothing
//! Out: as `free_rev` leaves them in neutral; no flags read
//!
//! Caller: `sub_43B635` (0x43B7B2), every physics step.
//! Self-test: `physics::engine_torque`'s (`test_engine_floats`), the converted version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::physics::engine_torque::g;
use crate::physics::free_rev::free_rev;
use crate::screens::harness::Left;

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn free_rev_in_neutral() -> Left {
    if g(0x71A1AC) != 0 {
        return Left::nothing();
    }
    unsafe { free_rev() }
}
