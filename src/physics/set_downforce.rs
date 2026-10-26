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
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;

unsafe fn at(a: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(a, "physics::set_downforce's operand")
}

unsafe fn put(a: u32, v: u32) {
    crate::pointers::write_or_end::<u32>(a, v, "physics::set_downforce's operand")
}

/// The Rust core on the variables at `v`.
pub unsafe fn set_downforce_at(v: &[u32; 6]) {
    unsafe {
        if at(v[0]) != 0 {
            return;
        }
        let (load, negative) = square_times_plus_times(
            at(v[1]),
            PHYSICS.downforce_factor.bits(),
            at(v[2]),
            PHYSICS.gravity.bits(),
        );
        put(v[3], load);
        put(v[4], negative);
        put(v[5], minus_one());
    }
}

fn game() -> [u32; 6] {
    [
        CAR.rear_wheels_off.addr() as u32,
        CAR.local_velocity[2].addr() as u32,
        PHYSICS.downforce_spec.addr() as u32,
        PHYSICS.downforce.addr() as u32,
        PHYSICS.forces[4].force[1].addr() as u32,
        PHYSICS.forces[4].point[2].addr() as u32,
    ]
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn set_downforce() {
    unsafe { set_downforce_at(&game()) }
}
