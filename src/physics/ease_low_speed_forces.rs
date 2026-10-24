//! `physics::ease_low_speed_forces` (0x440CF3): `flt_61475C` 1.0; when |`g_carSpeed`| is
//! below `flt_613E64` (20; an unordered compare counts as below), `flt_61475C` =
//! (|speed| / `flt_613E64` + `flt_613E68`) x `flt_613E6C` and the two forces
//! `flt_614248`, `flt_61424C` times it.
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller's next is an `fld`)
//!
//! Caller: `sub_43DFB1` (0x43E523).

use crate::framework::{game_addr, game_read, rva};
use crate::math::x87;
use crate::math::x87_run::{abs_compare, abs_div_add_mul};

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

/// The core: the factor, the speed and the two forces at these places.
pub unsafe fn ease_low_speed_forces_at(factor: u32, speed: u32, forces: [u32; 2]) {
    unsafe {
        put(factor, 0x3F80_0000);
        let limit = k(0x613E64);
        if abs_compare(at(speed), limit) & 0x100 == 0 {
            return;
        }
        put(
            factor,
            abs_div_add_mul(at(speed), limit, k(0x613E68), k(0x613E6C)),
        );
        for f in forces {
            put(f, x87::mul(at(f), at(factor)));
        }
    }
}

fn game() -> (u32, u32, [u32; 2]) {
    let g = |a: usize| game_addr(rva(a)) as u32;
    (g(0x61475C), g(0x71A184), [g(0x614248), g(0x61424C)])
}

/// The Rust version (the exact one, for `physics::drive_wheels_step`).
pub unsafe fn ease_low_speed_forces() {
    let (factor, speed, forces) = game();
    unsafe { ease_low_speed_forces_at(factor, speed, forces) }
}
