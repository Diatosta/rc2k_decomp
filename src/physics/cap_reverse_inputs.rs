//! `physics::cap_reverse_inputs` (0x46514A): in reverse (`g_carGear` -1) while the car's
//! speed (`g_carSpeed`) is not below 0 (an unordered compare counts as below), the two
//! floats `flt_614234` and `flt_614238` held to at most `flt_5C5D3C` (2.0; one that is
//! below it, or unordered, is left).
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller's next is a call)
//!
//! Caller: `sub_43DFB1` (0x43E519).
//! Self-test: a copy of the original (its four variables the test's) against the converted
//! version (compares and copies of floats only: exact at any precision):
//! gears, speeds about 0 and NaN, values about the cap.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_addr, game_read, rva};
use rc2k_core::physics::drive_forces::cap_reverse;

const CAP_RVA: usize = rva(0x5C5D3C); // flt_5C5D3C, 2.0
const ZERO_RVA: usize = rva(0x5C5D90); // flt_5C5D90, 0.0

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The converted version (`rc2k_core::physics::drive_forces`) on the gear, the speed and
/// the two values at these places.
// Converted: `rc2k_core::physics::drive_forces::cap_reverse`.
pub unsafe fn cap_reverse_inputs_at(gear: u32, speed: u32, values: [u32; 2]) {
    unsafe {
        if at(gear) != u32::MAX {
            return; // the speed and the values not read, as the original
        }
        let f = |a: u32| f32::from_bits(at(a));
        let zero = f32::from_bits(game_read(ZERO_RVA));
        let cap = f32::from_bits(game_read(CAP_RVA));
        let Some(held) = cap_reverse(at(gear), f(speed), zero, [f(values[0]), f(values[1])], cap)
        else {
            return;
        };
        for (v, h) in values.iter().zip(held) {
            if at(*v) != h.to_bits() {
                (*v as *mut u32).write_unaligned(h.to_bits());
            }
        }
    }
}

fn game() -> (u32, u32, [u32; 2]) {
    let g = |a: usize| game_addr(rva(a)) as u32;
    (g(0x71A1AC), g(0x71A184), [g(0x614234), g(0x614238)])
}

/// The Rust version (compares and copies of floats only: the same at any precision).
pub unsafe fn cap_reverse_inputs() {
    let (gear, speed, values) = game();
    unsafe { cap_reverse_inputs_at(gear, speed, values) }
}
