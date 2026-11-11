//! `physics::damage_step` (0x4AB5D6): the car's damage and wear model for this physics
//! step: `physics::damage_factors`, `physics::damage_wear`, `physics::damage_gauges`
//! (its only code: three calls and `retn`).
//!
//! Caller: `physics::car_physics_step` (0x43B952), every physics step.

use crate::physics::car_store::Car;
use crate::physics::store::DamageModel;
use crate::race::work_store::RaceWork;

pub fn damage_step(car: &Car, damage: &DamageModel, work: &RaceWork) {
    // The steps' arithmetic is written for rounding to nearest, the game's mode here.
    debug_assert!(crate::math::x87::rounds_to_nearest());
    crate::physics::damage_factors::steps(car, damage);
    crate::physics::damage_wear::steps(car, damage);
    crate::physics::damage_gauges::steps(car, damage, work);
}
