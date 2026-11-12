//! `physics::keep_motion` (0x440B7C): copies the car's `local_velocity` and `g_carSpeed` (three
//! dwords) into `kept_motion`.
//!
//! Caller: `physics::car_physics_step`, every physics step.

use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;

/// Bit copies of floats into words.
pub fn keep_motion(car: &Car, tables: &PhysicsTables) {
    for (kept, v) in tables.kept_motion.iter().zip(&car.local_velocity) {
        kept.set(v.bits());
    }
}
