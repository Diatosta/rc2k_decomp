//! `physics::count_start_second` (0x436AC4): `g_car` +0x88 (`stage_time`) up by one when
//! +0x26C (`tyre_change_state`) is -1 or 0, the start count +0x1C is 0, the timer +0x14 less
//! 0x320 is not below 0 (by its sign) and +0xA8 (`finish_flag`) is 0. A reading: time
//! counted once the start is behind (`physics::false_start`'s fields).
//!
//! Returns: the timer less 0x320 when it got that far
//!
//! Caller: `physics::race_timers` (four calls).

use crate::physics::car_store::Car;
use rc2k_core::physics::start::CarStart;

// Converted: `rc2k_core::physics::start::CarStart::count_start_second`.
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
