//! `physics::count_start_second` (0x436AC4): `g_car` +0x88 (`unk_71A060`) up by one when
//! +0x26C (`unk_71A244`) is -1 or 0, the start count +0x1C is 0, the timer +0x14 less
//! 0x320 is not below 0 (by its sign) and +0xA8 (`unk_71A080`) is 0. A reading: time
//! counted once the start is behind (`physics::false_start`'s fields).
//!
//! In:  nothing
//! Out: eax the timer less 0x320 when it got that far; the rest kept; no flags read
//!
//! Caller: `sub_436790` (four calls).

use crate::framework::{game_addr, rva};
use crate::physics::cells::{get, put_changed};
use rc2k_core::physics::start::CarStart;

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The variables (IDA addresses).
const VARIABLES: [usize; 5] = [0x71A244, 0x719FF4, 0x719FEC, 0x71A080, 0x71A060];

/// The Rust version on the variables' addresses.
// Converted: `rc2k_core::physics::start::CarStart::count_start_second`.
unsafe fn core(v: &[u32]) {
    unsafe {
        let was = CarStart {
            state: get(v[0]),
            count: get(v[1]),
            timer: get(v[2]),
            finished: get(v[3]),
            time: get(v[4]),
            ..CarStart::default()
        };
        let mut c = was;
        c.count_start_second();
        put_changed(v[4], was.time, c.time);
    }
}

/// The Rust version.
pub unsafe fn count_start_second() {
    unsafe { core(&game()) }
}
