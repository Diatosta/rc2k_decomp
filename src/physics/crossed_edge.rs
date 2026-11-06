//! `physics::crossed_edge` (0x45D737): when a probe point crossed a face's edge
//! (`dword_6D3B84` not -1): its three floats from `unk_71AAC8[n]` .. into `unk_71A1FC`
//! ..; `physics::restore_saved_car`; `physics::crossed_edge_contact`; `unk_71A208`
//! raised to |`flt_6D3108`|, `unk_71A20C` = 1, `unk_71A1CC` = 0x32;
//! `physics::feel_ground`.
//!
//! In:  nothing
//! Out: the registers as the calls leave them; no flags read (callers go on)
//!
//! Callers: `sub_43B635` (every physics step), `sub_43BDD1`. Refuses (before anything) a
//! probe point past 0x3F (the tables' room), and what its calls refuse (what was written
//! put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::crossed_edge_contact::check_probe;
use crate::race::owner::Race;
use crate::race::owner_physics::Physics;

/// The steps, not guarded (the probe checked before).
// Converted: `rc2k_core::physics::walls::{NONE_CROSSED, raised_hit}`.
pub fn steps(race: &Race) -> Result<(), Refused> {
    let physics_store = &race.physics;
    let n = physics_store.contact.crossed_probe.get();
    if n == rc2k_core::physics::walls::NONE_CROSSED {
        return Ok(());
    }
    let k = n as usize;
    CAR.contact_point[0].set_bits(CAR.probe_point_x[k].bits());
    CAR.contact_point[1].set_bits(CAR.probe_point_y[k].bits());
    CAR.contact_point[2].set_bits(CAR.probe_point_z[k].bits());
    { crate::physics::restore_saved_car::restore_saved_car(&race.physics) }?;
    crate::physics::crossed_edge_contact::steps(race)?;
    if let Some(hit) = rc2k_core::physics::walls::raised_hit(
        f32::from_bits(physics_store.contact.contact_speed_length.bits()),
        f32::from_bits(CAR.contact_depth.bits()),
    ) {
        CAR.contact_depth.set_bits(hit.to_bits());
    }
    CAR.contact_flag.set(1);
    CAR.wall_hit_timer.set(0x32);
    crate::physics::feel_ground::feel_ground(race)
}

/// The probe checked (-1 is none).
pub fn check(physics: &Physics) -> Result<(), Refused> {
    if physics.contact.crossed_probe.get() == rc2k_core::physics::walls::NONE_CROSSED {
        return Ok(());
    }
    check_probe(physics)
}

/// The Rust version. The registers it leaves are read by none of its callers.
pub fn crossed_edge(race: &Race) -> Result<(), Refused> {
    check(&race.physics)?;
    steps(race)
}
