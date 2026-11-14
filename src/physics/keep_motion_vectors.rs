//! Unless `contact_step` is set, the three dwords of the car's `velocity` (the vector
//! `physics::store_motion_length` measures) go as bits into `kept_vectors`, and those of `spin`
//! into the following three.
//!
//! Caller: `physics::car_physics_step`, every physics step.

use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;

/// Bit copies of floats into words.
pub fn keep_motion_vectors(car: &Car, tables: &PhysicsTables) {
    if tables.contact_step.get() != 0 {
        return;
    }
    let kept = &tables.kept_vectors;
    for k in 0..3 {
        kept[k].set(car.velocity[k].get());
        kept[4 + k].set(car.spin[k].get());
    }
}
