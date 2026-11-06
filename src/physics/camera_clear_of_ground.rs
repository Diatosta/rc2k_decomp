//! `physics::camera_clear_of_ground` (0x438DD2): whether the camera is a whole unit
//! (0x10000) or more above the ground under it (`unk_71A680` - `dword_639828`, by the
//! sign of the `cmp`; `physics/camera_ground.rs`): CF clear when found and so high, else
//! CF.
//!
//! In:  nothing
//! Out: CF; eax as the steps leave it, the rest as `count_ground_probe` leaves them (the
//!      caller reads CF)
//!
//! Caller: `sub_43907F` (three). Refuses what `count_ground_probe` refuses.
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
pub fn camera_clear_of_ground(race: &Race) -> Result<bool, Refused> {
    let mut clear = probe_at(
        race,
        crate::race::owner::race().shared.car.camera_position[0].get(),
        crate::race::owner::race().shared.car.camera_position[2].get(),
        &crate::race::owner::race().shared.car.camera_ground_faces[0],
    )?;
    if clear {
        clear = is_clear(over(
            crate::race::owner::race().shared.car.camera_position[1].get(),
            race.physics.ground.ground_height.get(),
        ));
    }
    Ok(clear)
}
