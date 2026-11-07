//! `physics::camera_over_ground` (0x438CEC): whether there is ground under the camera
//! and a half-unit (0x8000) to either side across its heading (`unk_71A688`,
//! `math::turn_point_xz` of (-0x8000, y, 0) and (0x8000, y, 0)), each probe from its own
//! last face (`unk_71AEC8`, `unk_71AECC`, `unk_71AED0`; `physics/camera_ground.rs`).
//! CF clear when all three are found; the first not found sets CF and ends it.
//!
//! Caller: `physics::camera_avoid_ground` (four). Refuses what `count_ground_probe` refuses.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::turn_point_xz::turn_point_xz;
use crate::physics::camera_ground::probe_at;
use crate::race::owner::Race;
use rc2k_core::physics::camera_ground::SIDES;

/// The Rust version: whether the camera and both sides are over ground (CF clear). The
/// registers it leaves are read by none of its callers.
// Converted: `rc2k_core::physics::camera_ground::SIDES`.
pub fn camera_over_ground(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    if !probe_at(
        race,
        race.shared.car.camera_position[0].get(),
        race.shared.car.camera_position[2].get(),
        &race.shared.car.camera_ground_faces[0],
        scene,
    )? {
        return Ok(false);
    }
    for (side, slot) in SIDES
        .into_iter()
        .zip(&race.shared.car.camera_ground_faces[1..])
    {
        race.course.course.ground_face_last.set(slot.get());
        // The y (the original's ebx, a leftover) only goes through the scratch point,
        // which nothing reads after: 0 here.
        let [x, _, z, _] = turn_point_xz(side as i32, 0, 0, race.shared.car.camera_heading.get());
        if !probe_at(
            race,
            x.wrapping_add(race.shared.car.camera_position[0].get()),
            z.wrapping_add(race.shared.car.camera_position[2].get()),
            slot,
            scene,
        )? {
            return Ok(false);
        }
    }
    Ok(true)
}
