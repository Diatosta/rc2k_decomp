//! `physics::ease_low_speed_forces` (0x440CF3): `low_speed_share` 1.0; when |`g_carSpeed`| is
//! below `low_speed_limit` (20; an unordered compare counts as below), `low_speed_share` =
//! (|speed| / `low_speed_limit` + `low_speed_add`) x `low_speed_scale` and the two forces
//! `drive_forces`, `flt_61424C` times it.
//!
//! Caller: `physics::drive_wheels_step` (0x43E523).

use crate::math::x87;
use crate::math::x87::abs_compare;
use crate::math::x87_float as bh;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::session::Cell32;

fn at(a: &dyn Cell32) -> u32 {
    a.bits()
}

fn put(a: &dyn Cell32, v: u32) {
    a.set_bits(v);
}

/// The core: the factor, the speed and the two forces at these places.
pub fn ease_low_speed_forces_at(
    tables: &PhysicsTables,
    factor: &dyn Cell32,
    speed: &dyn Cell32,
    forces: [&dyn Cell32; 2],
) {
    put(factor, 0x3F80_0000);
    let limit = tables.low_speed_limit.bits();
    if abs_compare(at(speed), limit) & 0x100 == 0 {
        return;
    }
    put(
        factor,
        bh::st(
            (bh::ld(at(speed)).abs() / bh::ld(limit) + bh::ld(tables.low_speed_add.bits()))
                * bh::ld(tables.low_speed_scale.bits()),
        ),
    );
    for f in forces {
        put(f, x87::mul(at(f), at(factor)).to_bits());
    }
}

/// The exact version, for `physics::drive_wheels_step`.
pub fn ease_low_speed_forces(car: &Car, tables: &PhysicsTables) {
    ease_low_speed_forces_at(
        tables,
        &tables.low_speed_share,
        &car.local_velocity[2],
        [&tables.drive_forces[0], &tables.drive_forces[1]],
    );
}
