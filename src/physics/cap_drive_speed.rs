//! `physics::cap_drive_speed` (0x440DF3): a slip allowance `drive_speed_figure` =
//! `drive_speed_cap` / (|`g_carSpeed`| + `drive_speed_cap_a`) - `drive_speed_cap_b`, held at 0 or more (an
//! unordered one counts as below); `g_carDriveSpeed` held to at most the speed plus it
//! (through `scratch`; a drive speed below that, or unordered, is left).
//!
//! Caller: `physics::drive_wheels_step` (0x43E0DD).

use crate::math::x87;
use crate::math::x87::over_abs_plus_minus;
use crate::physics::car_store::Car;
use crate::physics::flow::FloatView;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::tables_store::PhysicsTables;
use std::cmp::Ordering;

/// C0 of comparing `a` with `b`: below, or unordered.
fn below(a: f32, b: f32) -> bool {
    !matches!(a.partial_cmp(&b), Some(Ordering::Greater | Ordering::Equal))
}

/// The cap of this step: the allowance over the forward speed, the speed plus it (in the
/// scratch cell), and the first drive speed held to that.
pub fn cap_drive_speed(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) {
    let allowance = &tables.drive_speed_figure;
    let speed = &car.local_velocity[2];
    let scratch = &scene.drawing.scene.get().scratch;
    let drive_speed = &car.drive_speed_a;
    let zero = f32::from_bits(ZERO);
    allowance.set(over_abs_plus_minus(
        &tables.drive_speed_cap,
        speed.get(),
        &tables.drive_speed_cap_a,
        &tables.drive_speed_cap_b,
    ));
    if below(allowance.get(), zero) {
        allowance.set(x87::loaded(zero));
    }
    scratch.set_float(x87::add(allowance.get(), speed.get()));
    if !below(drive_speed.get(), scratch.float()) {
        drive_speed.set(x87::loaded(scratch.float()));
    }
}
