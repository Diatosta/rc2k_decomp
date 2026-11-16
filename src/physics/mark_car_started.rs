//! Sets `started_stand_timer` to 1 when it is 0 and `damage_failed` is not 0 (`started` after
//! `moving`: the car has begun to move).

use crate::physics::car_store::Car;
use rc2k_core::physics::start::CarStart;

// Core: `rc2k_core::physics::start::CarStart::mark_started`.
pub fn mark_car_started(car: &Car) {
    let was = CarStart {
        started: car.started_stand_timer.get(),
        moving: car.damage_failed.get(),
        ..CarStart::default()
    };
    let mut c = was;
    c.mark_started();
    crate::physics::cells::set_changed(&car.started_stand_timer, was.started, c.started);
}
