//! `physics::camera_clear_of_ground` (0x438DD2): whether the camera is a whole unit
//! (0x10000) or more above the ground under it (`unk_71A680` - `ground_height`, by the
//! sign of the `cmp`; `physics/camera_ground.rs`): CF clear when found and so high, else
//! CF.
//!
//! Caller: `physics::camera_avoid_ground` (three). Refuses what `count_ground_probe` refuses.

use crate::errors::Refused;
use crate::physics::camera_ground::probe_at;
use crate::physics::step_families::CameraGroundInputs;
use rc2k_core::physics::camera_ground::{is_clear, over};

/// Whether the camera is clear of the ground (CF clear).
// Converted: `rc2k_core::physics::camera_ground::{over, is_clear}`.
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
