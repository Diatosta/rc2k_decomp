//! The camera's ground checks: `physics::camera_low_switch`, `physics::camera_over_ground` and
//! `physics::camera_clear_of_ground`. Each probes the ground under a point through
//! `physics::count_ground_probe`, starting from the face found there last
//! (`CarBlock::camera_ground_faces`) and keeping the face found (`ground_slot`). The names of the
//! camera variables (`camera_view` .. `kept_focus_point`: mode, place, turns) are a reading.
//!
//! This file holds the shared steps.

use crate::errors::Refused;
use crate::physics::step_families::CameraGroundInputs;

/// One probe at (x, z) from the face kept at `slot`; true when found (the face found
/// kept there).
pub(super) fn probe_at(
    c: &CameraGroundInputs<'_>,
    x: u32,
    z: u32,
    slot: &mut u32,
) -> Result<bool, Refused> {
    {
        let mut search = c.search.borrow_mut();
        search.last = *slot;
        search.x = x;
        search.z = z;
    }
    {
        crate::physics::count_ground_probe::count_ground_probe(
            c.camera, c.probe, c.search, c.course,
        )
    }?;
    let search = c.search.borrow();
    if search.found == 0 {
        return Ok(false);
    }
    *slot = search.slot;
    Ok(true)
}
