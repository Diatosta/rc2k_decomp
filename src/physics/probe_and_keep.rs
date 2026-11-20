//! `physics::probe_ground`, then its three whole-number lists (x, y, z; 0xC0 dwords) and the probe
//! points' faces (`probe_point_face`, 0x40 dwords) are copied: the probe lists as they were before
//! the step (`saved_probe_x` ..), read by `crossed_edge_contact`.
//!
//! Refuses what `probe_ground` refuses (nothing written).

use crate::errors::Refused;
use crate::physics::probe_ground::probe_ground_at;
use crate::physics::step_families::GroundInputs;

/// The steps.
fn probe_and_keep_at(c: &GroundInputs<'_>) -> Result<(), Refused> {
    let contact = c.contact;
    probe_ground_at(
        &crate::physics::probe_ground::ProbeInputs::of_ground(c),
        c.car,
        &crate::physics::ground_query::GroundQuery::of_physics(
            c.ground,
            c.probe,
            c.course.course.get(),
            &crate::race::scene_points::no_own(),
        ),
        c.contact,
    )?;
    for k in 0..64 {
        contact.saved_probe_x[k].set(contact.probe_x[k].get());
    }
    for k in 0..64 {
        contact.saved_probe_y[k].set(contact.probe_y[k].get());
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
