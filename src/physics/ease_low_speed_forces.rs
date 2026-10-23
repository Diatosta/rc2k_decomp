//! `physics::ease_low_speed_forces` (0x440CF3): `flt_61475C` 1.0; when |`g_carSpeed`| is
//! below `flt_613E64` (20; an unordered compare counts as below), `flt_61475C` =
//! (|speed| / `flt_613E64` + `flt_613E68`) x `flt_613E6C` and the two forces
//! `flt_614248`, `flt_61424C` times it.
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller's next is an `fld`)
//!
//! Caller: `sub_43DFB1` (0x43E523).
//! Self-test: a copy of the original (its four variables the test's) against the converted
//! version, speeds by lot about the limit, exact at double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::math::x87;
use crate::math::x87_run::{abs_compare, abs_div_add_mul};
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;
use rc2k_core::physics::drive_forces::{eased, low_speed_factor};

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

/// The converted version (`rc2k_core::physics::drive_forces`, plain floats) on the
/// factor, the speed and the two forces at these places.
// Converted: `rc2k_core::physics::drive_forces::{low_speed_factor, eased}`.
pub unsafe fn ease_low_speed_forces_plain_at(factor: u32, speed: u32, forces: [u32; 2]) {
    unsafe {
        let f = |a: u32| f32::from_bits(at(a));
        let kf = |a: usize| f32::from_bits(k(a));
        put(factor, 0x3F80_0000);
        let Some(by) = low_speed_factor(f(speed), kf(0x613E64), kf(0x613E68), kf(0x613E6C)) else {
            return;
        };
        put(factor, by.to_bits());
        let eased = eased([f(forces[0]), f(forces[1])], by);
        put(forces[0], eased[0].to_bits());
        put(forces[1], eased[1].to_bits());
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

fn rust(_: &Regs) -> Result<Left, Refused> {
    let (factor, speed, forces) = game();
    unsafe { ease_low_speed_forces_plain_at(factor, speed, forces) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
