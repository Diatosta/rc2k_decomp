//! `physics::cap_reverse_inputs` (0x46514A): in reverse (`g_carGear` -1) while the car's
//! speed (`g_carSpeed`) is not below 0 (an unordered compare counts as below), the two
//! floats `flt_614234` and `flt_614238` held to at most `flt_5C5D3C` (2.0; one that is
//! below it, or unordered, is left).
//!
//! Caller: `physics::drive_wheels_step` (0x43E519).

use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::session::Cell32;
use rc2k_core::physics::drive_forces::cap_reverse;

fn at(a: &dyn Cell32) -> u32 {
    a.bits()
}

/// The converted version (`rc2k_core::physics::drive_forces`) on the gear, the speed and
/// the two values at these places.
// Converted: `rc2k_core::physics::drive_forces::cap_reverse`.
pub fn cap_reverse_inputs_at(
    course: &Course,
    gear: &dyn Cell32,
    speed: &dyn Cell32,
    values: [&dyn Cell32; 2],
) {
    if at(gear) != u32::MAX {
        return; // the speed and the values not read, as the original
    }
    let f = |a: &dyn Cell32| f32::from_bits(at(a));
    let zero = f32::from_bits(course.numbers.zero.bits());
    let cap = f32::from_bits(course.numbers.two.bits());
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
pub fn cap_reverse_inputs(race: &Race) {
    cap_reverse_inputs_at(
        &race.course,
        &race.shared.car.gear,
        &race.shared.car.local_velocity[2],
        [
            &race.shared.physics_tables.wheel_slips[0],
            &race.shared.physics_tables.wheel_slips[1],
        ],
    );
}
