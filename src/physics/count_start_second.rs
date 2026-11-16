//! Adds one to the car's `stage_time` when `tyre_change_state` is -1 or 0, the start count is 0,
//! the timer less 0x320 is not below 0 and `finish_flag` is 0. A reading: time counted once the
//! start is behind (`physics::false_start`'s fields).
//!
//! Returns the timer less 0x320 when it got that far.
//!
//! Caller: `physics::race_timers`.

use crate::physics::car_store::Car;
use rc2k_core::physics::start::CarStart;

// Core: `rc2k_core::physics::start::CarStart::count_start_second`.
pub fn count_start_second(car: &Car) {
    let was = CarStart {
        state: car.tyre_change_state.get(),
        count: car.false_start_count.get(),
        timer: car.run_ticks.get(),
        finished: car.finish_flag.get(),
        time: car.stage_time.get(),
        ..CarStart::default()
    };
    let mut c = was;
    c.count_start_second();
    if c.time != was.time {
        car.stage_time.set(c.time);
    }
}
