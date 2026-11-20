//! The car's damage and wear model for this physics step: `physics::damage_factors`,
//! `physics::damage_wear`, `physics::damage_gauges`.

use crate::errors::BadInput;
use crate::physics::car_store::Car;
use crate::physics::store::DamageModel;
use crate::race::work_store::RaceWork;

pub(super) fn damage_step(
    car: &Car,
    damage: &DamageModel,
    work: &RaceWork,
) -> Result<(), BadInput> {
    // The steps' arithmetic is written for rounding to nearest, the mode the damage model runs in.
    debug_assert!(crate::math::x87::rounds_to_nearest());
    crate::physics::damage_factors::steps(car, damage);
    crate::physics::damage_wear::steps(car, damage)?;
    crate::physics::damage_gauges::steps(car, damage, work);
    Ok(())
}
