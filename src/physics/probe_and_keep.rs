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
    probe_ground(c, c.car)?;
    let mut probes = c.probes.borrow_mut();
    probes.saved_probe_x = probes.probe_x;
    probes.saved_probe_z = probes.probe_z;
    for k in 0..0x40 {
        probes.saved_probe_faces[k] = c.car.probe_point_face[k].get();
    }
    Ok(())
}

pub(super) fn probe_and_keep(c: &GroundInputs<'_>) -> Result<(), Refused> {
    probe_and_keep_at(c)
}
