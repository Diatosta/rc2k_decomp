//! `physics::add_drive_forces` (0x440C7B): a factor (`flt_613EE4`, or with
//! `g_carDriveKind` set `flt_613EE8`, the first then stored in `dword_62400C`) times the
//! sum of the four floats at `flt_614284` (80 bits) into `flt_6140F4`; then
//! `flt_614248` += `flt_614234` x it x `flt_6140B8` and `flt_61424C` += `flt_614238` x it
//! x `flt_6140B8`.
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller's next is a call)
//!
//! Caller: `sub_43DFB1` (0x43E51E).

use crate::math::x87;
use crate::math::x87_run::mul2_plus;
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::scene::draw_store::DRAW;

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The core on the variables' addresses.
pub unsafe fn add_drive_forces_at(v: &[u32]) {
    unsafe {
        let mut factor = PHYSICS.drive_force_factor[0].bits();
        if at(v[1]) != 0 {
            put(v[0], x87::loaded(factor));
            factor = PHYSICS.drive_force_factor[1].bits();
        }
        put(
            v[6],
            x87::sum4_times(at(v[2]), at(v[3]), at(v[4]), at(v[5]), factor),
        );
        let scale = PHYSICS.drive_shares[2].bits();
        put(v[9], mul2_plus(at(v[7]), at(v[6]), scale, at(v[9])));
        put(v[10], mul2_plus(at(v[8]), at(v[6]), scale, at(v[10])));
    }
}

fn game() -> Vec<u32> {
    vec![
        DRAW.scratch.addr() as u32,
        CAR.drive_kind.addr() as u32,
        PHYSICS.wheel_loads[0].addr() as u32,
        PHYSICS.wheel_loads[1].addr() as u32,
        PHYSICS.wheel_loads[2].addr() as u32,
        PHYSICS.wheel_loads[3].addr() as u32,
        PHYSICS.drive_force_scale.addr() as u32,
        PHYSICS.wheel_slips[0].addr() as u32,
        PHYSICS.wheel_slips[1].addr() as u32,
        PHYSICS.drive_forces[0].addr() as u32,
        PHYSICS.drive_forces[1].addr() as u32,
    ]
}

/// The Rust version (the exact one, for `physics::drive_wheels_step`).
pub unsafe fn add_drive_forces() {
    unsafe { add_drive_forces_at(&game()) }
}
