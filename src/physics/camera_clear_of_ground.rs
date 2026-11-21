//! Whether the camera is a whole unit or more above the ground under it
//! (`Car.camera_position[1]` less `ground_height`); false when no ground is found.
//!
//! Refuses what `count_ground_probe` refuses.

use crate::errors::Refused;
use crate::physics::camera_ground::probe_at;
use crate::physics::step_families::CameraGroundInputs;
use rc2k_core::physics::camera_ground::{is_clear, over};

/// Whether the camera is clear of the ground.
// Core: `rc2k_core::physics::camera_ground::{over, is_clear}`.
pub(super) fn camera_clear_of_ground(c: &CameraGroundInputs<'_>) -> Result<bool, Refused> {
    let mut clear = probe_at(
        c,
        c.car.camera_position.bits_at(0),
        c.car.camera_position.bits_at(2),
        &c.car.camera_ground_faces[0],
    )?;
    if clear {
        clear = is_clear(over(
            c.car.camera_position.bits_at(1),
            c.search.borrow().height,
        ));
    }
    Ok(clear)
}
