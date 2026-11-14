//! The camera's ground checks: `physics::camera_low_switch`, `physics::camera_over_ground` and
//! `physics::camera_clear_of_ground`. Each probes the ground under a point through
//! `physics::count_ground_probe`, starting from the face found there last
//! (`CarBlock::camera_ground_faces`) and keeping the face found (`ground_slot`). The names of the
//! camera variables (`camera_view` .. `kept_focus_point`: mode, place, turns) are a reading.
//!
//! This file holds the shared steps.

use crate::errors::Refused;
use crate::physics::step_families::CameraGroundInputs;
use crate::session::PlainDword;

/// One probe at (x, z) from the face kept at `slot`; true when found (the face found
/// kept there).
pub fn probe_at(
    c: &CameraGroundInputs<'_>,
    x: u32,
    z: u32,
    slot: &PlainDword,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    c.course.course.get().ground_face_last.set(slot.get());
    c.ground.ground_x.set(x);
    c.ground.ground_z.set(z);
    {
        crate::physics::count_ground_probe::count_ground_probe(
            c.camera, c.probe, c.ground, c.course, scene,
        )
    }?;
    if c.ground.ground_found.get() == 0 {
        return Ok(false);
    }
    slot.set(c.course.course.get().ground_slot.get());
    Ok(true)
}
