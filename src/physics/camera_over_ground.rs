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
use crate::physics::step_families::CameraGroundInputs;
use rc2k_core::physics::camera_ground::SIDES;

/// Whether the camera and both sides are over ground (CF clear).
// Converted: `rc2k_core::physics::camera_ground::SIDES`.
pub fn camera_over_ground(
    engine_tables: &crate::scene::engine_tables_store::EngineTables,
    c: &CameraGroundInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    if !probe_at(
        c,
        c.car.camera_position[0].get(),
        c.car.camera_position[2].get(),
        &c.car.camera_ground_faces[0],
        scene,
    )? {
        return Ok(false);
    }
    for (side, slot) in SIDES.into_iter().zip(&c.car.camera_ground_faces[1..]) {
        c.course.course.get().ground_face_last.set(slot.get());
        // The y (a leftover value in the original) only goes through the scratch point,
        // which nothing reads after: 0 here.
        let [x, _, z, _] =
            rotate_point_about_y(engine_tables, side as i32, 0, 0, c.car.camera_heading.get());
        if !probe_at(
            c,
            x.wrapping_add(c.car.camera_position[0].get()),
            z.wrapping_add(c.car.camera_position[2].get()),
            slot,
            scene,
        )? {
            return Ok(false);
        }
    }
    Ok(true)
}
