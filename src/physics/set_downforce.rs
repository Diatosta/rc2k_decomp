//! `physics::set_downforce` (0x43F508): unless `dword_71A224` is set: `flt_614174` =
//! (`g_carSpeed`² `flt_613FB4` + `dword_61476C`) `flt_613FC8` (gravity, by
//! `physics::apply_patches`), `flt_6147F4` its negative, `flt_614804` = -1. By the look
//! of it the car's weight with downforce growing with the speed's square (a reading).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B728), every physics step.

use crate::framework::{game_addr, game_read, rva};
use crate::math::x87_run::{minus_one, square_times_plus_times};

/// The variables (IDA addresses): the switch, the speed, the offset, and the three
/// written.
const VARIABLES: [usize; 6] = [0x71A224, 0x71A184, 0x61476C, 0x614174, 0x6147F4, 0x614804];
const FACTOR_RVA: usize = rva(0x613FB4); // flt_613FB4
const GRAVITY_RVA: usize = rva(0x613FC8); // flt_613FC8

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core on the variables at `v`.
pub unsafe fn set_downforce_at(v: &[u32; 6]) {
    unsafe {
        if at(v[0]) != 0 {
            return;
        }
        let (load, negative) = square_times_plus_times(
            at(v[1]),
            game_read(FACTOR_RVA),
            at(v[2]),
            game_read(GRAVITY_RVA),
        );
        put(v[3], load);
        put(v[4], negative);
        put(v[5], minus_one());
    }
}

fn game() -> [u32; 6] {
    VARIABLES.map(|x| game_addr(rva(x)) as u32)
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn set_downforce() {
    unsafe { set_downforce_at(&game()) }
}
