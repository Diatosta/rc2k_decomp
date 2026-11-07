//! `physics::keep_motion_vectors` (0x43BD41): unless `dword_614148` is set, the three
//! dwords at `dword_71A170` (the vector `physics::store_motion_length` measures) into
//! `dword_614BCC` .. `D4`, and those at `dword_71A15C` into `dword_614BDC` .. `E4` (eax
//! pushed around).
//!
//! Caller: `sub_43B635` (0x43B89B), every physics step.

use crate::race::owner::Race;
use crate::session::Cell32;

fn core(v: &[&dyn Cell32]) {
    if v[0].bits() != 0 {
        return;
    }
    for k in 0..6 {
        v[k + 7].set_bits(v[k + 1].bits());
    }
}

fn game(race: &Race) -> Vec<&dyn Cell32> {
    vec![
        &race.shared.physics_tables.contact_step as &dyn Cell32,
        &race.shared.car.velocity[0],
        &race.shared.car.velocity[1],
        &race.shared.car.velocity[2],
        &race.shared.car.spin[0],
        &race.shared.car.spin[1],
        &race.shared.car.spin[2],
        &race.shared.physics_tables.kept_vectors[0],
        &race.shared.physics_tables.kept_vectors[1],
        &race.shared.physics_tables.kept_vectors[2],
        &race.shared.physics_tables.kept_vectors[4],
        &race.shared.physics_tables.kept_vectors[5],
        &race.shared.physics_tables.kept_vectors[6],
    ]
}

/// The Rust version.
pub fn keep_motion_vectors(race: &Race) {
    core(&game(race));
}
