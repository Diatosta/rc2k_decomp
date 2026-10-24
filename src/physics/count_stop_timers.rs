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

use crate::framework::{game_addr, game_read, rva};
use rc2k_core::physics::motion::stop_timers;

/// The variables (IDA addresses): three counters, the speed, the result.
const VARIABLES: [usize; 5] = [0x71A078, 0x71A07C, 0x71A080, 0x71A19C, 0x71A088];

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The core on the variables' addresses (a compare of floats only: the same at any
/// precision).
// Converted: `rc2k_core::physics::motion::stop_timers`.
pub unsafe fn count_stop_timers_at(v: &[u32]) {
    unsafe {
        let one = f32::from_bits(game_read(rva(0x5C5D38)));
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
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn count_stop_timers() {
    unsafe { count_stop_timers_at(&game()) }
}
