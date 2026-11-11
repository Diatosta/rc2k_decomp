//! `physics::keep_motion_vectors` (0x43BD41): unless `contact_step` is set, the three dwords of
//! the car's `velocity` (the vector `physics::store_motion_length` measures) go into
//! `kept_vectors`, and those of `spin` into the following three.
//!
//! Caller: `physics::car_physics_step`, every physics step.

use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::session::Cell32;

fn core(v: &[&dyn Cell32]) {
    if v[0].bits() != 0 {
        return;
    }
    for k in 0..6 {
        v[k + 7].set_bits(v[k + 1].bits());
    }
}

fn game<'a>(car: &'a Car, tables: &'a PhysicsTables) -> Vec<&'a dyn Cell32> {
    vec![
        &tables.contact_step as &dyn Cell32,
        &car.velocity[0],
        &car.velocity[1],
        &car.velocity[2],
        &car.spin[0],
        &car.spin[1],
        &car.spin[2],
        &tables.kept_vectors[0],
        &tables.kept_vectors[1],
        &tables.kept_vectors[2],
        &tables.kept_vectors[4],
        &tables.kept_vectors[5],
        &tables.kept_vectors[6],
    ]
}

pub fn keep_motion_vectors(car: &Car, tables: &PhysicsTables) {
    core(&game(car, tables));
}
