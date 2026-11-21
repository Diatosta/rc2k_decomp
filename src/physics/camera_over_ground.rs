//! Whether there is ground under the camera and under points half a unit (0x8000) to either side
//! of it across its heading; each probe starts from its own last face (`camera_ground_faces`).
//! True only when all three are found; the first one not found ends it.
//!
//! Refuses what `count_ground_probe` refuses.

use crate::errors::Refused;
use crate::math::rotate_point_about_y::rotate_point_about_y;
use crate::physics::camera_ground::probe_at;
use crate::physics::step_families::CameraGroundInputs;
use rc2k_core::physics::camera_ground::SIDES;

/// Whether the camera and both sides are over ground.
// Core: `rc2k_core::physics::camera_ground::SIDES`.
pub(super) fn camera_over_ground(
    engine_tables: &crate::math::engine_tables::EngineTables,
    c: &CameraGroundInputs<'_>,
) -> Result<bool, Refused> {
    if !probe_at(
        c,
        c.car.camera_position.bits_at(0),
        c.car.camera_position.bits_at(2),
        &c.car.camera_ground_faces[0],
    )? {
        return Ok(false);
    }
    for (side, slot) in SIDES.into_iter().zip(&c.car.camera_ground_faces[1..]) {
        c.search.borrow_mut().last = slot.get();
        // The y only goes through the scratch point, which nothing reads after: 0 here.
        let [x, _, z, _] =
            rotate_point_about_y(engine_tables, side as i32, 0, 0, c.car.camera_heading.get());
        if !probe_at(
            c,
            x.wrapping_add(c.car.camera_position.bits_at(0)),
            z.wrapping_add(c.car.camera_position.bits_at(2)),
            slot,
        )? {
            return Ok(false);
        }
    }
    Ok(true)
}
