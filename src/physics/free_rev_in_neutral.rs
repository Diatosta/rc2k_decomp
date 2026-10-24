//! `physics::free_rev_in_neutral` (0x43EA54): `physics::free_rev` in neutral.
//!
//! In:  nothing
//! Out: as `free_rev` leaves them in neutral; no flags read
//!
//! Caller: `sub_43B635` (0x43B7B2), every physics step.

use crate::physics::engine_torque::g;
use crate::physics::free_rev::free_rev;

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn free_rev_in_neutral() {
    if g(0x71A1AC) != 0 {
        return;
    }
    unsafe { free_rev() }
}
