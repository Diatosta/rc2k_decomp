//! Whether a car in the stage (`driver_kind` not 0) is
//! within 3 checkpoints of the given one: |`checkpoint` - given| less 4 is negative.

use crate::physics::car_store::Car;
use rc2k_core::physics::line_places::car_near;

pub fn near_at(car: &Car, checkpoint: u32) -> bool {
    car_near(
        car.driver_kind.get() != 0,
        || car.checkpoint.get(),
        checkpoint,
    )
    .is_some_and(|n| n.1)
}
