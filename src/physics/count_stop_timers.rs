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
use rc2k_core::physics::motion::stop_timers;

unsafe fn at(a: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(a, "physics::count_stop_timers's operand")
}

unsafe fn put(a: u32, v: u32) {
    crate::pointers::write_or_end::<u32>(a, v, "physics::count_stop_timers's operand")
}

/// The core on the variables' addresses (a compare of floats only: the same at any
/// precision).
// Converted: `rc2k_core::physics::motion::stop_timers`.
pub unsafe fn count_stop_timers_at(v: &[u32]) {
    unsafe {
        let one = f32::from_bits(NUMBERS.one.bits());
        let mut counters = [at(v[0]), at(v[1]), at(v[2])];
        let result = stop_timers(&mut counters, f32::from_bits(at(v[3])), one);
        for (k, c) in counters.iter().enumerate() {
            if *c != at(v[k]) {
                put(v[k], *c);
            }
        }
        if let Some(result) = result {
            put(v[4], result);
        }
    }
}

fn game() -> Vec<u32> {
    vec![
        CAR.started_stand_timer.addr() as u32,
        CAR.go_stand_timer.addr() as u32,
        CAR.finish_flag.addr() as u32,
        CAR.speed.addr() as u32,
        CAR.stop_state.addr() as u32,
    ]
}

/// The Rust version.
pub unsafe fn count_stop_timers() {
    unsafe { count_stop_timers_at(&game()) }
}
