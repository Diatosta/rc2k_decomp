//! Copies the car's `local_velocity` and car speed (three dwords) into `kept_motion`.

use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;

/// Bit copies of floats into words.
pub(super) fn keep_motion(car: &Car, tables: &PhysicsTables) {
    for (kept, v) in tables.kept_motion.iter().zip(&car.local_velocity) {
        kept.set(v.get());
    }
}
