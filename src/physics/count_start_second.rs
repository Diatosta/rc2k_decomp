//! `physics::count_start_second` (0x436AC4): `g_car` +0x88 (`stage_time`) up by one when
//! +0x26C (`tyre_change_state`) is -1 or 0, the start count +0x1C is 0, the timer +0x14 less
//! 0x320 is not below 0 (by its sign) and +0xA8 (`finish_flag`) is 0. A reading: time
//! counted once the start is behind (`physics::false_start`'s fields).
//!
//! Returns: the timer less 0x320 when it got that far
//!
//! Caller: `physics::race_timers` (four calls).

use crate::physics::car_store::Car;
use crate::session::Cell32;
use rc2k_core::physics::start::CarStart;

fn game(car: &Car) -> Vec<&dyn Cell32> {
    vec![
        &car.tyre_change_state as &dyn Cell32,
        &car.false_start_count,
        &car.run_ticks,
        &car.finish_flag,
        &car.stage_time,
    ]
}

/// On the variables' cells.
// Converted: `rc2k_core::physics::start::CarStart::count_start_second`.
fn core(v: &[&dyn Cell32]) {
    let was = CarStart {
        state: v[0].bits(),
        count: v[1].bits(),
        timer: v[2].bits(),
        finished: v[3].bits(),
        time: v[4].bits(),
        ..CarStart::default()
    };
    let mut c = was;
    c.count_start_second();
    crate::physics::cells::set_changed(v[4], was.time, c.time);
}

pub fn count_start_second(car: &Car) {
    core(&game(car));
}
