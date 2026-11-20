//! In reverse while the car's speed is not below 0 (an unordered compare counts as below), the
//! two `wheel_slips` floats are held to at most `two` (2.0; a lower or unordered one is left).

use crate::physics::car_store::Car;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use rc2k_core::physics::drive_forces::cap_reverse;

/// The converted version (`rc2k_core::physics::drive_forces::cap_reverse`) on the gear,
/// the forward speed and the two wheel slips; compares and copies of floats only, the same
/// at any precision.
pub(super) fn cap_reverse_inputs(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    let gear = car.gear.get();
    if gear != -1 {
        return; // the speed and the values are not read
    }
    let values = &tables.wheel_slips;
    let zero = f32::from_bits(ZERO);
    let cap = numbers.two.get();
    let speed = car.local_velocity[2].get();
    let Some(held) = cap_reverse(gear, speed, zero, [values[0].get(), values[1].get()], cap) else {
        return;
    };
    for (v, h) in values.iter().zip(held) {
        if v.get().to_bits() != h.to_bits() {
            v.set(h);
        }
    }
}
