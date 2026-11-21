//! `physics::probe_ground`, then its x and z whole-number lists and the probe
//! points' faces (`probe_point_face`, 0x40 dwords) are copied: the probe lists as they were before
//! the step (`saved_probe_x`, `saved_probe_z`), read by `crossed_edge_contact`.
//!
//! Refuses what `probe_ground` refuses (nothing written).

use crate::errors::Refused;
use crate::physics::probe_ground::probe_ground;
use crate::physics::step_families::GroundInputs;

/// The steps.
fn probe_and_keep_at(c: &GroundInputs<'_>) -> Result<(), Refused> {
    let contact = c.contact;
    probe_ground(c, c.car)?;
    for k in 0..64 {
        contact.saved_probe_x[k].set(contact.probe_x[k].get());
    }
    for k in 0..64 {
        contact.saved_probe_z[k].set(contact.probe_z[k].get());
    }
    for k in 0..0x40 {
        contact.saved_probe_faces[k].set(c.car.probe_point_face[k].get());
    }
    Ok(())
}

pub(super) fn probe_and_keep(c: &GroundInputs<'_>) -> Result<(), Refused> {
    probe_and_keep_at(c)
}
