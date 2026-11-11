//! `physics::count_stop_timers` (0x436978): three counters of the work car (`g_car`
//! +0xA0, +0xA4, +0xA8: `started_stand_timer` .. `finish_flag`) in turn. One that is not 0: at 300
//! `stop_state` (+0xB0) is set (-1 for the first two, 2 for the third) and it returns;
//! else while +0x1C4 (`speed`) is below `one` (1.0; unordered counts as
//! below) it goes up, and on reaching 300 (unsigned) the same. A reading: the car has
//! stood still too long after a start or a finish.
//!
//! Caller: `physics::race_timers` (four calls).

use crate::physics::car_store::Car;
use crate::race::numbers_store::Numbers;
use crate::session::Cell32;
use rc2k_core::physics::motion::stop_timers;

/// The core on the variables' cells (a compare of floats only: the same at any
/// precision).
// Converted: `rc2k_core::physics::motion::stop_timers`.
pub fn count_stop_timers_at(numbers: &Numbers, v: &[&dyn Cell32]) {
    let one = f32::from_bits(numbers.one.bits());
    let mut counters = [v[0].bits(), v[1].bits(), v[2].bits()];
    let result = stop_timers(&mut counters, f32::from_bits(v[3].bits()), one);
    for (k, c) in counters.iter().enumerate() {
        if *c != v[k].bits() {
            v[k].set_bits(*c);
        }
    }
    if let Some(result) = result {
        v[4].set_bits(result);
    }
}

fn game(car: &Car) -> Vec<&dyn Cell32> {
    vec![
        &car.started_stand_timer as &dyn Cell32,
        &car.go_stand_timer,
        &car.finish_flag,
        &car.speed,
        &car.stop_state,
    ]
}

pub fn count_stop_timers(car: &Car, numbers: &Numbers) {
    count_stop_timers_at(numbers, &game(car));
}
