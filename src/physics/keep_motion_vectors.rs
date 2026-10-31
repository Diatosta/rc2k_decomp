//! `physics::keep_motion_vectors` (0x43BD41): unless `dword_614148` is set, the three
//! dwords at `dword_71A170` (the vector `physics::store_motion_length` measures) into
//! `dword_614BCC` .. `D4`, and those at `dword_71A15C` into `dword_614BDC` .. `E4` (eax
//! pushed around).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B89B), every physics step.

use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::session::Cell32;

fn core(v: &[&dyn Cell32]) {
    if v[0].bits() != 0 {
        return;
    }
    for k in 0..6 {
        v[k + 7].set_bits(v[k + 1].bits());
    }
}

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &PHYSICS.contact_step as &dyn Cell32,
        &CAR.velocity[0],
        &CAR.velocity[1],
        &CAR.velocity[2],
        &CAR.spin[0],
        &CAR.spin[1],
        &CAR.spin[2],
        &PHYSICS.kept_vectors[0],
        &PHYSICS.kept_vectors[1],
        &PHYSICS.kept_vectors[2],
        &PHYSICS.kept_vectors[4],
        &PHYSICS.kept_vectors[5],
        &PHYSICS.kept_vectors[6],
    ]
}

/// The Rust version.
pub fn keep_motion_vectors() {
    core(&game());
}
