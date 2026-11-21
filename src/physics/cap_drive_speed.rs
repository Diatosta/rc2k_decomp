//! Caps the drive speed: the slip allowance `drive_speed_figure` = `drive_speed_cap` /
//! (|car speed| + `drive_speed_cap_a`) - `drive_speed_cap_b`, held at 0 or more (an unordered one
//! counts as below); the car's drive speed is held to at most the speed plus it (a lower or
//! unordered drive speed is left).

use crate::math::x87;
use crate::math::x87::over_abs_plus_minus;
use crate::physics::car_store::Car;
use crate::physics::figures::{DRIVE_SPEED_CAP, DRIVE_SPEED_CAP_A, DRIVE_SPEED_CAP_B};
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::tables_store::PhysicsTables;
use std::cmp::Ordering;

/// C0 of comparing `a` with `b`: below, or unordered.
fn below(a: f32, b: f32) -> bool {
    !matches!(a.partial_cmp(&b), Some(Ordering::Greater | Ordering::Equal))
}

/// The cap of this step: the allowance over the forward speed, the speed plus it, and the first
/// drive speed held to that.
pub fn cap_drive_speed(car: &Car, tables: &PhysicsTables) {
    let allowance = &tables.drive_speed_figure;
    let speed = &car.local_velocity[2];
    let drive_speed = &car.drive_speed_a;
    let zero = f32::from_bits(ZERO);
    allowance.set(over_abs_plus_minus(
        DRIVE_SPEED_CAP,
        speed.get(),
        DRIVE_SPEED_CAP_A,
        DRIVE_SPEED_CAP_B,
    ));
    if below(allowance.get(), zero) {
        allowance.set(x87::loaded(zero));
    }
    let cap = x87::add(allowance.get(), speed.get());
    if !below(drive_speed.get(), cap) {
        drive_speed.set(x87::loaded(cap));
    }
}
