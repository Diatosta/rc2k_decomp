//! Counts three counters of the work car (`started_stand_timer` .. `finish_flag`) in turn. One
//! that is not 0: at 300 `stop_state` is set (-1 for the first two, 2 for the third) and it
//! returns; otherwise while `speed` is below `one` (1.0; unordered counts as below) it goes up,
//! and on reaching 300 (unsigned) the same. A reading: the car has stood still too long after a
//! start or a finish.

use crate::memory::session::PlainDword;
use crate::physics::car_store::Car;
use crate::race::numbers_store::Numbers;
use rc2k_core::physics::motion::stop_timers;

/// The stand timers and the finish flag counted on while the car stands (a compare of
/// floats only: the same at any precision).
// Core: `rc2k_core::physics::motion::stop_timers`.
pub(super) fn count_stop_timers(car: &Car, numbers: &Numbers) {
    let one = numbers.one.get();
    let cells = [
        &car.started_stand_timer,
        &car.go_stand_timer,
        &car.finish_flag,
    ];
    let mut counters = cells.map(PlainDword::get);
    let result = stop_timers(&mut counters, car.speed.get(), one);
    for (cell, c) in cells.iter().zip(counters) {
        if c != cell.get() {
            cell.set(c);
        }
    }
    if let Some(result) = result {
        car.stop_state.set(result);
    }
}
