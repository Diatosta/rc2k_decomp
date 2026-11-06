//! `physics::set_downforce` (0x43F508): unless `dword_71A224` is set: `flt_614174` =
//! (`g_carSpeed`² `flt_613FB4` + `dword_61476C`) `flt_613FC8` (gravity, by
//! `physics::apply_patches`), `flt_6147F4` its negative, `flt_614804` = -1. By the look
//! of it the car's weight with downforce growing with the speed's square (a reading).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B728), every physics step.

use crate::math::x87_run::{minus_one, square_times_plus_times};

/// The Rust version.
pub fn set_downforce() {
    if crate::race::owner::race().shared.car.rear_wheels_off.get() != 0 {
        return;
    }
    let (load, negative) = square_times_plus_times(
        crate::race::owner::race().shared.car.local_velocity[2].bits(),
        crate::race::owner::race()
            .shared
            .physics_tables
            .downforce_factor
            .bits(),
        crate::race::owner::race()
            .shared
            .physics_tables
            .downforce_spec
            .bits(),
        crate::race::owner::race()
            .shared
            .physics_tables
            .gravity
            .bits(),
    );
    crate::race::owner::race()
        .shared
        .physics_tables
        .downforce
        .set_bits(load);
    crate::race::owner::race().shared.physics_tables.forces[4].force[1].set_bits(negative);
    crate::race::owner::race().shared.physics_tables.forces[4].point[2].set_bits(minus_one());
}
