//! `drive::car_near_checkpoint` (0x4375F2): whether a car in the stage (`driver_kind` not 0) is
//! within 3 checkpoints of the given one: |`checkpoint` - given| less 4 is negative.
//!
//! Takes: a car block, a checkpoint
//!
//! Caller: `drive::checkpoint_taken` (four cars).

use crate::physics::car_store::Car;
use rc2k_core::physics::line_places::car_near;

/// The Rust core: whether the car block `car` is near `checkpoint`.
// Converted: `rc2k_core::physics::line_places::car_near`.
pub fn near_at(car: &Car, checkpoint: u32) -> bool {
    car_near(
        car.driver_kind.get() != 0,
        || car.checkpoint.get(),
        checkpoint,
    )
    .is_some_and(|n| n.1)
}
