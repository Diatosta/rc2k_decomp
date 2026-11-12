//! `drive::checkpoint_taken` (0x4375A5): whether a checkpoint is no place to start a car: CF when
//! it is within 8 of the stage's start or end (`g_checkpointCount` - 8; the sign of the
//! differences) or `drive::car_near_checkpoint` finds one of the four cars near it (the car blocks).
//!
//! Takes: the checkpoint
//!
//! Caller: `drive::start_cars_behind`.

use crate::drive::car_near_checkpoint::near_at;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use rc2k_core::physics::line_places;

/// The Rust core: the four cars `cars`, the count `count`, the checkpoint
/// `checkpoint`: whether it is taken.
// Converted: `rc2k_core::physics::line_places::checkpoint_taken`.
pub fn taken_at(cars: [&Car; 4], count: u32, checkpoint: u32) -> bool {
    line_places::checkpoint_taken(checkpoint, count, || {
        cars.iter().any(|car| near_at(car, checkpoint))
    })
}

/// Whether `checkpoint` is taken.
pub fn checkpoint_taken(race: &Race, checkpoint: u32) -> bool {
    let cars = [0, 1, 2, 3].map(|n| &race.car_blocks().cars[n]);
    taken_at(
        cars,
        race.shared.vars.get().checkpoint_count.get(),
        checkpoint,
    )
}
