//! `physics::add_drive_forces` (0x440C7B): a factor (`drive_force_factor`, or with
//! `g_carDriveKind` set `flt_613EE8`, the first then stored in `scratch`) times the
//! sum of the four floats at `wheel_loads` (80 bits) into `drive_force_scale`; then
//! `drive_forces` += `wheel_slips` x it x `flt_6140B8` and `flt_61424C` += `flt_614238` x it
//! x `flt_6140B8`.
//!
//! Caller: `physics::drive_wheels_step` (0x43E51E).

use crate::math::x87;
use crate::math::x87::mul2_plus;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::session::Cell32;

fn put(a: &dyn Cell32, v: u32) {
    a.set_bits(v);
}

/// The core on the variables' cells.
pub fn add_drive_forces_at(tables: &PhysicsTables, v: &[&dyn Cell32]) {
    let mut factor = tables.drive_force_factor[0].bits();
    if v[1].bits() != 0 {
        v[0].set_bits(x87::loaded(factor));
        factor = tables.drive_force_factor[1].bits();
    }
    put(
        v[6],
        x87::sum4_times(v[2].bits(), v[3].bits(), v[4].bits(), v[5].bits(), factor),
    );
    let scale = tables.drive_shares[2].bits();
    v[9].set_bits(mul2_plus(v[7].bits(), v[6].bits(), scale, v[9].bits()));
    v[10].set_bits(mul2_plus(v[8].bits(), v[6].bits(), scale, v[10].bits()));
}

fn game<'a>(
    car: &'a Car,
    tables: &'a PhysicsTables,
    scene: &'a crate::scene::owner::Scene,
) -> Vec<&'a dyn Cell32> {
    vec![
        &scene.drawing.scene.scratch as &dyn Cell32,
        &car.drive_kind,
        &tables.wheel_loads[0],
        &tables.wheel_loads[1],
        &tables.wheel_loads[2],
        &tables.wheel_loads[3],
        &tables.drive_force_scale,
        &tables.wheel_slips[0],
        &tables.wheel_slips[1],
        &tables.drive_forces[0],
        &tables.drive_forces[1],
    ]
}

/// The exact version, for `physics::drive_wheels_step`.
pub fn add_drive_forces(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) {
    add_drive_forces_at(tables, &game(car, tables, scene));
}
