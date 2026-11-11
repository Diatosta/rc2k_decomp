//! `physics::mark_car_started` (0x436AA7): `g_car` +0xA0 (`started_stand_timer`) set to 1 when
//! it is 0 and +0x29C (`damage_failed`) is not (`started` after `moving`: the car has begun to move).
//!
//! Caller: `physics::race_timers` (four calls).

use crate::physics::car_store::Car;
use crate::session::Cell32;
use rc2k_core::physics::start::CarStart;

fn game(car: &Car) -> Vec<&dyn Cell32> {
    vec![&car.started_stand_timer as &dyn Cell32, &car.damage_failed]
}

/// On the variables' cells.
// Converted: `rc2k_core::physics::start::CarStart::mark_started`.
fn core(v: &[&dyn Cell32]) {
    let was = CarStart {
        started: v[0].bits(),
        moving: v[1].bits(),
        ..CarStart::default()
    };
    let mut c = was;
    c.mark_started();
    crate::physics::cells::set_changed(v[0], was.started, c.started);
}

pub fn mark_car_started(car: &Car) {
    core(&game(car));
}
