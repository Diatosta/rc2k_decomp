//! `physics::cap_reverse_inputs` (0x46514A): in reverse (`g_carGear` -1) while the car's
//! speed (`g_carSpeed`) is not below 0 (an unordered compare counts as below), the two
//! floats `flt_614234` and `flt_614238` held to at most `flt_5C5D3C` (2.0; one that is
//! below it, or unordered, is left).
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller's next is a call)
//!
//! Caller: `sub_43DFB1` (0x43E519).

use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::numbers_store::NUMBERS;
use rc2k_core::physics::drive_forces::cap_reverse;

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
        let zero = f32::from_bits(NUMBERS.zero.bits());
        let cap = f32::from_bits(NUMBERS.two.bits());
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
    (
        (CAR.gear.addr() as u32),
        (CAR.local_velocity[2].addr() as u32),
        [
            (PHYSICS.wheel_slips[0].addr() as u32),
            (PHYSICS.wheel_slips[1].addr() as u32),
        ],
    )
}

/// The Rust version (compares and copies of floats only: the same at any precision).
pub unsafe fn cap_reverse_inputs() {
    let (gear, speed, values) = game();
    unsafe { cap_reverse_inputs_at(gear, speed, values) }
}
