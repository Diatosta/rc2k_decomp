//! In gear, `engine_speed` = `drive_speed_a` over `gear_ratios[gear]`; with `drive_kind` 1 the two
//! driven speeds' sum times 0.5 (`half`) over it. Nothing in neutral.
//!
//! See `physics/gearing.rs`. Callers: the gearbox under `drive::race_step` (`physics::change_gear`,
//! `physics::manual_gearbox`, `physics::auto_gearbox`, `physics::drive_wheels_step`),
//! `physics::test_drive_run` (the car specification's run, under `drive::race_frame`), and
//! `debug::dyno`. Refuses (before writing) a gear outside -1 ..= 7 (the ratio table).

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::gearing::*;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;

/// The exact version, for the gearbox's Rust callers, on the game's variables.
pub fn revs_from_wheels(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
) -> Result<(), Refused> {
    revs_from_wheels_at(numbers, &GearPlaces::game(car, tables))
}
