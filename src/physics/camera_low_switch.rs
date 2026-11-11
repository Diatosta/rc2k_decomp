//! `physics::camera_low_switch` (0x438BF4): with `Car.crash_matrix[5]` below 0 (or unordered), no
//! switch under way (`camera_switch_active` 0) and the camera in mode 0 or 1 (`camera_view`): the
//! ground under the camera (`camera_position`, `Car.camera_position[2]`; `physics/camera_ground.rs`);
//! found and the camera less than 0x800 above it (`Car.camera_position[1]` - `ground_height`, by the
//! sign of the `cmp`): the switch begun: `camera_view_return` = the mode before (`camera_blend_view`)
//! plus 1, `camera_blend_view` = 4, `camera_switch_active` = 1, `camera_switch_share` = 0, `camera_switch_length` = 10,
//! `camera_switch_clock` = the clock (`step_clock`).
//!
//! Caller: `physics::update_camera`. Refuses what `count_ground_probe` refuses.

use crate::errors::Refused;
use crate::physics::camera_ground::probe_at;
use crate::physics::flow::{ZERO_BITS as ZERO, c0};
use crate::physics::step_families::CameraGroundInputs;
use rc2k_core::physics::camera_ground::{is_low, low_switch, low_switch_looks, over};

// Converted: `rc2k_core::physics::camera_ground::{low_switch_looks, is_low, low_switch}`.
pub fn camera_low_switch(
    c: &CameraGroundInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    if !low_switch_looks(
        c0(&c.car.crash_matrix[5], ZERO),
        c.car.camera_switch_active.get(),
        c.car.camera_view.get(),
    ) {
        return Ok(());
    }
    if !probe_at(
        c,
        c.car.camera_position[0].get(),
        c.car.camera_position[2].get(),
        &c.car.camera_ground_faces[0],
        scene,
    )? {
        return Ok(());
    }
    if !is_low(over(
        c.car.camera_position[1].get(),
        c.ground.ground_height.get(),
    )) {
        return Ok(());
    }
    let s = low_switch(c.car.camera_blend_view.get(), c.car.step_clock.get());
    c.car.camera_view_return.set(s.before);
    c.car.camera_blend_view.set(s.mode);
    c.car.camera_switch_active.set(s.switching);
    c.car.camera_switch_share.set_bits(s.w674);
    c.car.camera_switch_length.set(s.w670);
    c.car.camera_switch_clock.set(s.clock);
    Ok(())
}
