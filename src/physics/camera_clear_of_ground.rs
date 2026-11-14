//! Whether the camera is a whole unit or more above the ground under it
//! (`Car.camera_position[1]` less `ground_height`); false when no ground is found.
//!
//! Caller: `physics::camera_avoid_ground`. Refuses what `count_ground_probe` refuses.

use crate::errors::Refused;
use crate::physics::camera_ground::probe_at;
use crate::physics::step_families::CameraGroundInputs;
use rc2k_core::physics::camera_ground::{is_clear, over};

/// Whether the camera is clear of the ground.
// Core: `rc2k_core::physics::camera_ground::{over, is_clear}`.
pub fn camera_clear_of_ground(
    c: &CameraGroundInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    let mut clear = probe_at(
        c,
        c.car.camera_position[0].get(),
        c.car.camera_position[2].get(),
        &c.car.camera_ground_faces[0],
        scene,
    )?;
    if clear {
        clear = is_clear(over(
            c.car.camera_position[1].get(),
            c.ground.ground_height.get(),
        ));
    }
    Ok(clear)
}
