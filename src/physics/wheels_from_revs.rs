//! In gear, `drive_speed_a` = `engine_speed` times `gear_ratios[gear]`; with `drive_kind` 1
//! `drive_speed_b` the same. Nothing in neutral.
//!
//! See `physics/gearing.rs`. Refuses (before writing) a gear outside -1 ..= 7 (the ratio table).

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::gearing::*;
use crate::physics::tables_store::PhysicsTables;

/// The exact version, for the gearbox's Rust callers, on the game's variables.
pub fn wheels_from_revs(car: &Car, tables: &PhysicsTables) -> Result<(), Refused> {
    wheels_from_revs_at(&GearPlaces::game(car, tables))
}
