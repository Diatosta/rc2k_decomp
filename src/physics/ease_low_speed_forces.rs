//! `low_speed_share` = 1.0; when |car speed| is below `low_speed_limit` (20; an unordered
//! compare counts as below), `low_speed_share` = (|speed| / `low_speed_limit` +
//! `low_speed_add`) x `low_speed_scale` and the two `drive_forces` are multiplied by it.

use crate::math::cell_x87::CellX87;
use crate::math::x87;
use crate::math::x87::abs_compare;
use crate::physics::car_store::Car;
use crate::physics::figures::{LOW_SPEED_ADD, LOW_SPEED_LIMIT, LOW_SPEED_SCALE};
use crate::physics::tables_store::PhysicsTables;

/// The factor, the speed and the two forces; the exact version, for
/// `physics::drive_wheels_step`.
pub(super) fn ease_low_speed_forces(car: &Car, tables: &PhysicsTables) {
    let factor = &tables.low_speed_share;
    let speed = &car.local_velocity[2];
    factor.set(1.0);
    let limit = LOW_SPEED_LIMIT;
    if abs_compare(speed, limit) & 0x100 == 0 {
        return;
    }
    // The product is rounded to a float where it is stored.
    factor.st(
        (speed.ld().abs() / crate::math::x87_float::ld(LOW_SPEED_LIMIT)
            + crate::math::x87_float::ld(LOW_SPEED_ADD))
            * crate::math::x87_float::ld(LOW_SPEED_SCALE),
    );
    for f in &tables.drive_forces {
        f.set(x87::mul(f, factor));
    }
}
