//! `physics::mark_car_started` (0x436AA7): `g_car` +0xA0 (`unk_71A078`) set to 1 when
//! it is 0 and +0x29C (`unk_71A274`) is not (a reading: the car has got going).
//!
//! In:  nothing
//! Out: every register kept; no flags read (the callers return next)
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
const VARIABLES: [usize; 2] = [0x71A078, 0x71A274];

/// The Rust version on the variables' addresses.
// Converted: `rc2k_core::physics::start::CarStart::mark_started`.
unsafe fn core(v: &[u32]) {
    unsafe {
        let was = CarStart {
            started: get(v[0]),
            moving: get(v[1]),
            ..CarStart::default()
        };
        let mut c = was;
        c.mark_started();
        put_changed(v[0], was.started, c.started);
    }
}

/// The Rust version.
pub unsafe fn mark_car_started() {
    unsafe { core(&game()) }
}
