//! `physics::damage_step` (0x4AB5D6): the car's damage and wear model for this physics
//! step: `physics::damage_factors`, `physics::damage_wear`, `physics::damage_gauges`
//! (its only code: three calls and `retn`).
//!
//! Caller: `physics::car_physics_step` (0x43B952), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::race::owner::Race;

pub fn damage_step(race: &Race) {
    crate::physics::damage_factors::steps(race);
    crate::physics::damage_wear::steps(race);
    crate::physics::damage_gauges::steps(race);
}
