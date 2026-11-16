//! Adds the drive forces: a factor (`drive_force_factor`, or its second entry when
//! the car's drive kind is set) times the sum of the four wheel loads becomes `drive_force_scale`;
//! then `drive_forces[0]` and `drive_forces[1]` gain `wheel_slips[0]` and `wheel_slips[1]` times
//! that scale times `drive_shares[2]`.

use crate::math::x87;
use crate::math::x87::mul2_plus;
use crate::physics::car_store::Car;
use crate::physics::flow::FloatView;
use crate::physics::tables_store::PhysicsTables;

/// The forces' step.
pub fn add_drive_forces(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) {
    let mut factor = tables.drive_force_factor[0].get();
    if car.drive_kind.get() != 0 {
        scene
            .drawing
            .scene
            .get()
            .scratch
            .set_float(x87::loaded(factor));
        factor = tables.drive_force_factor[1].get();
    }
    let loads = &tables.wheel_loads;
    tables.drive_force_scale.set(x87::sum4_times(
        &loads[0], &loads[1], &loads[2], &loads[3], factor,
    ));
    let scale = tables.drive_shares[2].get();
    let forces = &tables.drive_forces;
    let it = &tables.drive_force_scale;
    forces[0].set(mul2_plus(&tables.wheel_slips[0], it, scale, &forces[0]));
    forces[1].set(mul2_plus(&tables.wheel_slips[1], it, scale, &forces[1]));
}
