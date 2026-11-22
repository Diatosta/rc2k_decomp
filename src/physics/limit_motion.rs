//! Holds the work car's motion to its limits. A motion part that is no number or infinite (its
//! exponent all ones) logs "*** INFINITY", "[car0.newxspd] =" and the three (5 decimals) and zeroes
//! the motion, its car-frame copy and the spin. Then the spin's parts are held to +-`spin_limit`
//! and the motion's (x, z, y) to +-`velocity_limit`; past `top_speed` (against the motion's
//! length, `physics::vector_length`) the motion and its copy are scaled down to it
//! (`physics::scale_vector`).
//!
//! The x87 compares are plain Rust over `X` (`math/x87_float.rs`).

use crate::logfile::log_float::log_float;
use crate::math::cell_x87::PlainX87;
use crate::math::matrix3::rounded_value;
use crate::math::x87_float::compare;
use crate::physics::car_store::Car;
use crate::physics::figures::{SPIN_LIMIT, VELOCITY_LIMIT};
use crate::physics::scale_vector::scale_vector_plain;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::vector_length::vector_length;
use std::cmp::Ordering;

/// The value at `v` held to +-`limit` (a float's bits).
fn hold(v: &mut f32, limit: u32) {
    let limit = crate::math::x87_float::ld(limit);
    if compare(v.ld(), limit) & 0x100 == 0 {
        v.st(limit);
    }
    let negated = -limit;
    if compare(v.ld(), negated) & 0x100 != 0 {
        v.st(negated);
    }
}

/// The exact version, for `physics::car_physics_step` and `physics::find_clear_place`.
pub fn limit_motion(car: &mut Car, tables: &PhysicsTables) {
    // Infinity or not a number (the float's exponent bits are tested).
    let bad = car.velocity.iter().any(|item| !item.is_finite());
    if bad {
        crate::logfile::log_write::log_nul(&c"*** INFINITY", false, 1);
        crate::logfile::log_write::log_nul(&c"[car0.newxspd] =", false, 1);
        for (k, &value) in car.velocity.iter().enumerate() {
            log_float(value, 5, k == 2, 1);
        }

        car.velocity = [0.0; 3];
        car.local_velocity = [0.0; 3];
        car.spin = [0.0; 3];
    }
    for spin in &mut car.spin {
        hold(spin, SPIN_LIMIT.to_bits());
    }
    for k in [0usize, 2, 1] {
        hold(&mut car.velocity[k], VELOCITY_LIMIT.to_bits());
    }
    let length = rounded_value(vector_length(&car.velocity));
    let top = tables.top_speed;
    let past =
        matches!(top.partial_cmp(&length), Some(Ordering::Less)) || top.is_nan() || length.is_nan();
    if past {
        let factor = crate::math::x87::div_80(top, length);
        scale_vector_plain(&mut car.velocity, factor);
        scale_vector_plain(&mut car.local_velocity, factor);
    }
}
