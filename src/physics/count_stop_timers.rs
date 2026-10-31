//! `physics::count_stop_timers` (0x436978): three counters of the work car (`g_car`
//! +0xA0, +0xA4, +0xA8: `unk_71A078` .. `unk_71A080`) in turn. One that is not 0: at 300
//! `unk_71A088` (+0xB0) is set (-1 for the first two, 2 for the third) and it returns;
//! else while +0x1C4 (`unk_71A19C`) is below `flt_5C5D38` (1.0; unordered counts as
//! below) it goes up, and on reaching 300 (unsigned) the same. A reading: the car has
//! stood still too long after a start or a finish.
//!
//! In:  nothing
//! Out: every register kept; no flags read (the callers go on with calls)
//!
//! Caller: `sub_436790` (four calls).

use crate::physics::car_store::CAR;
use crate::race::numbers_store::NUMBERS;
use crate::session::Cell32;
use rc2k_core::physics::motion::stop_timers;

/// The core on the variables' cells (a compare of floats only: the same at any
/// precision).
// Converted: `rc2k_core::physics::motion::stop_timers`.
pub fn count_stop_timers_at(v: &[&dyn Cell32]) {
    let one = f32::from_bits(NUMBERS.one.bits());
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

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &CAR.started_stand_timer as &dyn Cell32,
        &CAR.go_stand_timer,
        &CAR.finish_flag,
        &CAR.speed,
        &CAR.stop_state,
    ]
}

/// The Rust version.
pub fn count_stop_timers() {
    count_stop_timers_at(&game());
}
