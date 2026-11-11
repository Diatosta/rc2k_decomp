//! `physics::probe_and_keep` (0x45DC12): `physics::probe_ground`, then its three
//! whole-number lists (x, y, z at 0x6D1E14, 0xC0 dwords) copied to 0x6D2C14 and the probe
//! points' faces (`probe_point_face`, 0x40 dwords) to 0x6D2F14 (`rep movsd`): the probe lists
//! as they were before the step (`saved_probe_x` ..., read by `crossed_edge_contact`).
//!
//! Caller: `physics::car_physics_step` (0x43B700), every physics step; next a call (no flags read).
//! Refuses what `probe_ground` refuses (nothing written).

use crate::errors::Refused;
use crate::physics::probe_ground::probe_ground_at;
use crate::race::owner::Race;

/// The steps.
pub fn probe_and_keep_at(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let physics = &race.physics;
    probe_ground_at(
        race,
        &race.shared.car,
        scene,
        &crate::physics::ground_query::GroundQuery::of_physics(race),
        &race.physics.contact,
        &scene.drawing.scene.scratch,
    )?;
    for k in 0..64 {
        physics.contact.saved_probe_x[k].set(physics.contact.probe_x[k].get());
    }
    for k in 0..64 {
        physics.contact.saved_probe_y[k].set(physics.contact.probe_y[k].get());
    }
    for k in 0..64 {
        physics.contact.saved_probe_z[k].set(physics.contact.probe_z[k].get());
    }
    for k in 0..0x40 {
        physics.contact.saved_probe_faces[k].set(race.shared.car.probe_point_face[k].get());
    }
    Ok(())
}

pub fn probe_and_keep(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    probe_and_keep_at(race, scene)
}
