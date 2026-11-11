//! `physics::cap_drive_speed` (0x440DF3): a slip allowance `drive_speed_figure` =
//! `drive_speed_cap` / (|`g_carSpeed`| + `drive_speed_cap_a`) - `drive_speed_cap_b`, held at 0 or more (an
//! unordered one counts as below); `g_carDriveSpeed` held to at most the speed plus it
//! (through `scratch`; a drive speed below that, or unordered, is left).
//!
//! Caller: `physics::drive_wheels_step` (0x43E0DD).

use crate::math::x87;
use crate::math::x87::over_abs_plus_minus;
use crate::physics::car_store::Car;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::tables_store::PhysicsTables;
use crate::session::Cell32;
use std::cmp::Ordering;

/// C0 of comparing `a` with `b`: below, or unordered.
fn below(a: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

fn at(a: &dyn Cell32) -> u32 {
    a.bits()
}

fn put(a: &dyn Cell32, v: u32) {
    a.set_bits(v);
}

/// The places: the allowance, the speed, the scratch, the drive speed.
pub type CapPlaces<'a> = [&'a dyn Cell32; 4];

/// The core.
pub fn cap_drive_speed_at(tables: &PhysicsTables, v: CapPlaces<'_>) {
    let zero = ZERO;
    put(
        v[0],
        over_abs_plus_minus(
            tables.drive_speed_cap.bits(),
            at(v[1]),
            tables.drive_speed_cap_a.bits(),
            tables.drive_speed_cap_b.bits(),
        ),
    );
    if below(at(v[0]), zero) {
        put(v[0], x87::loaded(zero));
    }
    put(v[2], x87::add(at(v[0]), at(v[1])));
    if !below(at(v[3]), at(v[2])) {
        put(v[3], x87::loaded(at(v[2])));
    }
}

fn game<'a>(
    car: &'a Car,
    tables: &'a PhysicsTables,
    scene: &'a crate::scene::owner::Scene,
) -> CapPlaces<'a> {
    [
        &tables.drive_speed_figure,
        &car.local_velocity[2],
        &scene.drawing.scene.scratch,
        &car.drive_speed_a,
    ]
}

/// The exact version, for `physics::drive_wheels_step`.
pub fn cap_drive_speed(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) {
    cap_drive_speed_at(tables, game(car, tables, scene));
}
