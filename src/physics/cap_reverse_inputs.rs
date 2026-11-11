//! `physics::cap_reverse_inputs` (0x46514A): in reverse (`g_carGear` -1) while the car's
//! speed (`g_carSpeed`) is not below 0 (an unordered compare counts as below), the two
//! floats `wheel_slips` and `flt_614238` held to at most `two` (2.0; one that is
//! below it, or unordered, is left).
//!
//! Caller: `physics::drive_wheels_step` (0x43E519).

use crate::physics::car_store::Car;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::session::Cell32;
use rc2k_core::physics::drive_forces::cap_reverse;

fn at(a: &dyn Cell32) -> u32 {
    a.bits()
}

/// The converted version (`rc2k_core::physics::drive_forces`) on the gear, the speed and
/// the two values at these places.
// Converted: `rc2k_core::physics::drive_forces::cap_reverse`.
pub fn cap_reverse_inputs_at(
    numbers: &Numbers,
    gear: &dyn Cell32,
    speed: &dyn Cell32,
    values: [&dyn Cell32; 2],
) {
    if at(gear) != u32::MAX {
        return; // the speed and the values not read, as the original
    }
    let f = |a: &dyn Cell32| f32::from_bits(at(a));
    let zero = f32::from_bits(ZERO);
    let cap = f32::from_bits(numbers.two.bits());
    let Some(held) = cap_reverse(at(gear), f(speed), zero, [f(values[0]), f(values[1])], cap)
    else {
        return;
    };
    for (v, h) in values.iter().zip(held) {
        if at(*v) != h.to_bits() {
            v.set_bits(h.to_bits());
        }
    }
}

/// Compares and copies of floats only: the same at any precision.
pub fn cap_reverse_inputs(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    cap_reverse_inputs_at(
        numbers,
        &car.gear,
        &car.local_velocity[2],
        [&tables.wheel_slips[0], &tables.wheel_slips[1]],
    );
}
