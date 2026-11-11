//! `physics::camera_over_ground` (0x438CEC): whether there is ground under the camera and
//! under a point half a unit (0x8000) to either side of it across its heading
//! (`camera_heading`, `math::rotate_point_about_y` of (-0x8000, y, 0) and (0x8000, y, 0)), each probe
//! starting from its own last face (`camera_ground_faces`; `physics/camera_ground.rs`). Reports
//! that ground is found when all three are; the first not found ends it.
//!
//! Caller: `physics::camera_avoid_ground` (four). Refuses what `count_ground_probe` refuses.

use crate::errors::Refused;
use crate::math::rotate_point_about_y::rotate_point_about_y;
use crate::physics::camera_ground::probe_at;
use crate::race::owner::Race;
use rc2k_core::physics::camera_ground::SIDES;

/// Whether the camera and both sides are over ground (CF clear).
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
        // The y (a leftover value in the original) only goes through the scratch point,
        // which nothing reads after: 0 here.
        let [x, _, z, _] =
            rotate_point_about_y(side as i32, 0, 0, race.shared.car.camera_heading.get());
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
