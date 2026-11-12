//! `physics::ease_low_speed_forces` (0x440CF3): `low_speed_share` 1.0; when |`g_carSpeed`| is
//! below `low_speed_limit` (20; an unordered compare counts as below), `low_speed_share` =
//! (|speed| / `low_speed_limit` + `low_speed_add`) x `low_speed_scale` and the two forces
//! `drive_forces`, `PhysicsTables.drive_forces[1]` times it.
//!
//! Caller: `physics::drive_wheels_step` (0x43E523).

use crate::effects::cell_x87::CellX87;
use crate::math::x87;
use crate::math::x87::abs_compare;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;

/// The factor, the speed and the two forces; the exact version, for
/// `physics::drive_wheels_step`.
pub fn ease_low_speed_forces(car: &Car, tables: &PhysicsTables) {
    let factor = &tables.low_speed_share;
    let speed = &car.local_velocity[2];
    factor.set(1.0);
    let limit = &tables.low_speed_limit;
    if abs_compare(speed, limit) & 0x100 == 0 {
        return;
    }
    // The product is rounded to a float where it is stored (`fstp`).
    factor
        .st((speed.ld().abs() / limit.ld() + tables.low_speed_add.ld())
            * tables.low_speed_scale.ld());
    for f in &tables.drive_forces {
        f.set(x87::mul(f, factor));
    }
}
