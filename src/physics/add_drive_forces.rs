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
use crate::session::Cell32;

fn put(a: &dyn Cell32, v: u32) {
    a.set_bits(v);
}

/// The core on the variables' cells.
pub fn add_drive_forces_at(v: &[&dyn Cell32]) {
    let mut factor = PHYSICS.drive_force_factor[0].bits();
    if v[1].bits() != 0 {
        v[0].set_bits(x87::loaded(factor));
        factor = PHYSICS.drive_force_factor[1].bits();
    }
    put(
        v[6],
        x87::sum4_times(v[2].bits(), v[3].bits(), v[4].bits(), v[5].bits(), factor),
    );
    let scale = PHYSICS.drive_shares[2].bits();
    v[9].set_bits(mul2_plus(v[7].bits(), v[6].bits(), scale, v[9].bits()));
    v[10].set_bits(mul2_plus(v[8].bits(), v[6].bits(), scale, v[10].bits()));
}

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &DRAW.scratch as &dyn Cell32,
        &CAR.drive_kind,
        &PHYSICS.wheel_loads[0],
        &PHYSICS.wheel_loads[1],
        &PHYSICS.wheel_loads[2],
        &PHYSICS.wheel_loads[3],
        &PHYSICS.drive_force_scale,
        &PHYSICS.wheel_slips[0],
        &PHYSICS.wheel_slips[1],
        &PHYSICS.drive_forces[0],
        &PHYSICS.drive_forces[1],
    ]
}

/// The Rust version (the exact one, for `physics::drive_wheels_step`).
pub fn add_drive_forces() {
    add_drive_forces_at(&game());
}
