//! `physics::mark_car_started` (0x436AA7): `g_car` +0xA0 (`started_stand_timer`) set to 1 when
//! it is 0 and +0x29C (`damage_failed`) is not (`started` after `moving`: the car has begun to move).
//!
//! Caller: `physics::race_timers` (four calls).

use crate::physics::car_store::Car;
use rc2k_core::physics::start::CarStart;

// Converted: `rc2k_core::physics::start::CarStart::mark_started`.
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
