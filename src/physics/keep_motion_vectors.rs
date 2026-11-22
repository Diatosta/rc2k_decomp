//! Unless `contact_step` is set, the three dwords of the car's `velocity` (the vector
//! `physics::store_motion_length` measures) go as bits into `kept_vectors`, and those of `spin`
//! into the following three.

use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;

/// Bit copies of floats into words.
pub(super) fn keep_motion_vectors(car: &Car, tables: &mut PhysicsTables) {
    if tables.contact_step != 0 {
        return;
    }
    let kept = &mut tables.kept_vectors;
    for k in 0..3 {
        kept[k] = car.velocity.at(k);
        kept[4 + k] = car.spin.at(k);
    }
}
