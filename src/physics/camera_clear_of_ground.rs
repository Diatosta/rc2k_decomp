//! `physics::camera_clear_of_ground` (0x438DD2): whether the camera is a whole unit
//! (0x10000) or more above the ground under it (`unk_71A680` - `dword_639828`, by the
//! sign of the `cmp`; `physics/camera_ground.rs`): CF clear when found and so high, else
//! CF.
//!
//! Caller: `physics::camera_avoid_ground` (three). Refuses what `count_ground_probe` refuses.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::camera_ground::probe_at;
use crate::race::owner::Race;
use rc2k_core::physics::camera_ground::{is_clear, over};

/// The Rust version: whether the camera is clear of the ground (CF clear). The registers it
/// leaves are read by none of its callers.
// Converted: `rc2k_core::physics::camera_ground::{over, is_clear}`.
pub fn camera_clear_of_ground(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    let mut clear = probe_at(
        race,
        race.shared.car.camera_position[0].get(),
        race.shared.car.camera_position[2].get(),
        &race.shared.car.camera_ground_faces[0],
        scene,
    )?;
    if clear {
        clear = is_clear(over(
            race.shared.car.camera_position[1].get(),
            race.physics.ground.ground_height.get(),
        ));
    }
    Ok(clear)
}
