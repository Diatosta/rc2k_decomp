//! `physics::cap_drive_speed` (0x440DF3): a slip allowance `flt_614168` =
//! `flt_613E48` / (|`g_carSpeed`| + `flt_613E4C`) - `flt_613E50`, held at 0 or more (an
//! unordered one counts as below); `g_carDriveSpeed` held to at most the speed plus it
//! (through `dword_62400C`; a drive speed below that, or unordered, is left).
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller's next is a `mov`)
//!
//! Caller: `sub_43DFB1` (0x43E0DD).
//! Self-test: a copy of the original (its four variables the test's) against the converted
//! version: speeds and drive speeds by lot and about the cap, exact at double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::math::x87;
use crate::math::x87_run::over_abs_plus_minus;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;
use rc2k_core::physics::drive_forces::cap_drive_speed as cap_drive_speed_core;
use std::cmp::Ordering;

/// C0 of comparing `a` with `b`: below, or unordered.
fn below(a: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

/// The places: the allowance, the speed, the scratch, the drive speed.
pub type CapPlaces = [u32; 4];

/// The core.
pub unsafe fn cap_drive_speed_at(v: CapPlaces) {
    unsafe {
        let zero = k(0x5C5D90);
        put(
            v[0],
            over_abs_plus_minus(k(0x613E48), at(v[1]), k(0x613E4C), k(0x613E50)),
        );
        if below(at(v[0]), zero) {
            put(v[0], x87::loaded(zero));
        }
        put(v[2], x87::add(at(v[0]), at(v[1])));
        if !below(at(v[3]), at(v[2])) {
            put(v[3], x87::loaded(at(v[2])));
        }
    }
}

/// The converted version (`rc2k_core::physics::drive_forces`, plain floats).
// Converted: `rc2k_core::physics::drive_forces::cap_drive_speed`.
pub unsafe fn cap_drive_speed_plain_at(v: CapPlaces) {
    unsafe {
        let f = |a: u32| f32::from_bits(at(a));
        let kf = |a: usize| f32::from_bits(k(a));
        let (allowance, cap, drive) = cap_drive_speed_core(
            kf(0x613E48),
            f(v[1]),
            kf(0x613E4C),
            kf(0x613E50),
            kf(0x5C5D90),
            f(v[3]),
        );
        put(v[0], allowance.to_bits());
        put(v[2], cap.to_bits());
        if at(v[3]) != drive.to_bits() {
            put(v[3], drive.to_bits());
        }
    }
}

fn game() -> CapPlaces {
    let g = |a: usize| game_addr(rva(a)) as u32;
    [g(0x614168), g(0x71A184), g(0x62400C), g(0x71A190)]
}

/// The Rust version (the exact one, for `physics::drive_wheels_step`).
pub unsafe fn cap_drive_speed() {
    unsafe { cap_drive_speed_at(game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { cap_drive_speed_plain_at(game()) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
