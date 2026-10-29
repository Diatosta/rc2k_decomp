//! `physics::ease_low_speed_forces` (0x440CF3): `flt_61475C` 1.0; when |`g_carSpeed`| is
//! below `flt_613E64` (20; an unordered compare counts as below), `flt_61475C` =
//! (|speed| / `flt_613E64` + `flt_613E68`) x `flt_613E6C` and the two forces
//! `flt_614248`, `flt_61424C` times it.
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller's next is an `fld`)
//!
//! Caller: `sub_43DFB1` (0x43E523).

use crate::math::x87;
use crate::math::x87_run::{abs_compare, abs_div_add_mul};
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::session::Cell32;

fn at(a: &dyn Cell32) -> u32 {
    a.bits()
}

fn put(a: &dyn Cell32, v: u32) {
    a.set_bits(v)
}

/// The core: the factor, the speed and the two forces at these places.
pub fn ease_low_speed_forces_at(factor: &dyn Cell32, speed: &dyn Cell32, forces: [&dyn Cell32; 2]) {
    put(factor, 0x3F80_0000);
    let limit = PHYSICS.low_speed_limit.bits();
    if abs_compare(at(speed), limit) & 0x100 == 0 {
        return;
    }
    put(
        factor,
        abs_div_add_mul(
            at(speed),
            limit,
            PHYSICS.low_speed_add.bits(),
            PHYSICS.low_speed_scale.bits(),
        ),
    );
    for f in forces {
        put(f, x87::mul(at(f), at(factor)));
    }
}

/// The Rust version (the exact one, for `physics::drive_wheels_step`).
pub fn ease_low_speed_forces() {
    ease_low_speed_forces_at(
        &PHYSICS.low_speed_share,
        &CAR.local_velocity[2],
        [&PHYSICS.drive_forces[0], &PHYSICS.drive_forces[1]],
    )
}
