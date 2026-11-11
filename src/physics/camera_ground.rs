//! The camera's ground checks: `physics::camera_low_switch`
//! (0x438BF4), `physics::camera_over_ground` (0x438CEC), `physics::camera_clear_of_ground`
//! (0x438DD2). Each probes the ground under a point through `physics::count_ground_probe`
//! (called directly), starting from the face it found there last
//! (`CarBlock::camera_ground_faces`, into `ground_face_last`) and keeping the face found (`ground_slot`).
//! What the camera variables (`camera_view` .. `kept_focus_point`) are is a reading: the mode,
//! the place (`camera_position` ..), the turns (`camera_heading` ..).
//!
//! This file: the shared steps.

use crate::errors::Refused;
use crate::physics::step_families::CameraGroundInputs;

/// One probe at (x, z) from the face kept at `slot`; true when found (the face found
/// kept there).
pub fn probe_at(
    c: &CameraGroundInputs<'_>,
    x: u32,
    z: u32,
    slot: &crate::session::Dword,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    c.course.course.ground_face_last.set(slot.get());
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
    slot.set(c.course.course.ground_slot.get());
    Ok(true)
}
