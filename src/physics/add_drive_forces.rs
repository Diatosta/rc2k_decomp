//! `physics::add_drive_forces` (0x440C7B): a factor (`flt_613EE4`, or with
//! `g_carDriveKind` set `flt_613EE8`, the first then stored in `dword_62400C`) times the
//! sum of the four floats at `flt_614284` (80 bits) into `flt_6140F4`; then
//! `flt_614248` += `flt_614234` x it x `flt_6140B8` and `flt_61424C` += `flt_614238` x it
//! x `flt_6140B8`.
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller's next is a call)
//!
//! Caller: `sub_43DFB1` (0x43E51E).

use crate::framework::{game_addr, game_read, rva};
use crate::math::x87;
use crate::math::x87_run::mul2_plus;

/// The variables (IDA addresses), in the order the core takes them.
pub const VARIABLES: [usize; 11] = [
    0x62400C, 0x71A018, 0x614284, 0x614288, 0x61428C, 0x614290, 0x6140F4, 0x614234, 0x614238,
    0x614248, 0x61424C,
];

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

/// The core on the variables' addresses.
pub unsafe fn add_drive_forces_at(v: &[u32]) {
    unsafe {
        let mut factor = k(0x613EE4);
        if at(v[1]) != 0 {
            put(v[0], x87::loaded(factor));
            factor = k(0x613EE8);
        }
        put(
            v[6],
            x87::sum4_times(at(v[2]), at(v[3]), at(v[4]), at(v[5]), factor),
        );
        let scale = k(0x6140B8);
        put(v[9], mul2_plus(at(v[7]), at(v[6]), scale, at(v[9])));
        put(v[10], mul2_plus(at(v[8]), at(v[6]), scale, at(v[10])));
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact one, for `physics::drive_wheels_step`).
pub unsafe fn add_drive_forces() {
    unsafe { add_drive_forces_at(&game()) }
}
