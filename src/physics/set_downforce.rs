//! `physics::set_downforce` (0x43F508): unless `dword_71A224` is set: `flt_614174` =
//! (`g_carSpeed`² `flt_613FB4` + `dword_61476C`) `flt_613FC8` (gravity, by
//! `physics::apply_patches`), `flt_6147F4` its negative, `flt_614804` = -1. The car's
//! downward force: its weight plus downforce growing with the speed's square.
//!
//! Caller: `physics::car_physics_step` (0x43B728), every physics step.

use crate::math::x87_run::{minus_one, square_times_plus_times};
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;

pub fn set_downforce(car: &Car, tables: &PhysicsTables) {
    if car.rear_wheels_off.get() != 0 {
        return;
    }
    let (load, negative) = square_times_plus_times(
        car.local_velocity[2].bits(),
        tables.downforce_factor.bits(),
        tables.downforce_spec.bits(),
        tables.gravity.bits(),
    );
    tables.downforce.set_bits(load);
    tables.forces[4].force[1].set_bits(negative);
    tables.forces[4].point[2].set_bits(minus_one());
}
