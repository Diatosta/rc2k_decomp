//! Begins the switch to the low camera. Needs `Car.crash_matrix[5]` below 0 (or unordered), no
//! switch under way and the camera in mode 0 or 1; then the ground under the camera is probed,
//! and when it is found and the camera is less than 0x800 above it, the switch starts:
//! `camera_view_return` = the mode before plus 1, `camera_blend_view` = 4, `camera_switch_active`
//! = 1, `camera_switch_share` = 0, `camera_switch_length` = 10, `camera_switch_clock` = the step
//! clock.
//!
//! Refuses what `count_ground_probe` refuses.

use crate::errors::Refused;
use crate::memory::plain_floats::FloatRow;
use crate::physics::camera_ground::probe_at;
use crate::physics::car_store::Car;
use crate::physics::flow::{ZERO_BITS as ZERO, c0};
use crate::physics::step_families::CameraGroundInputs;
use rc2k_core::physics::camera_ground::{is_low, low_switch, low_switch_looks, over};

// Core: `rc2k_core::physics::camera_ground::{low_switch_looks, is_low, low_switch}`.
pub(super) fn camera_low_switch(c: &CameraGroundInputs<'_>, car: &mut Car) -> Result<(), Refused> {
    if !low_switch_looks(
        c0(car.crash_matrix.at(5), ZERO),
        car.camera_switch_active,
        car.camera_view,
    ) {
        return Ok(());
    }
    if !probe_at(
        c,
        car.camera_position.bits_at(0),
        car.camera_position.bits_at(2),
        &mut car.camera_ground_faces[0],
    )? {
        return Ok(());
    }
    if !is_low(over(
        car.camera_position.bits_at(1),
        c.search.borrow().height,
    )) {
        return Ok(());
    }
    let s = low_switch(car.camera_blend_view, car.step_clock);
    car.camera_view_return = s.before;
    car.camera_blend_view = s.mode;
    car.camera_switch_active = s.switching;
    car.camera_switch_share = f32::from_bits(s.switch_share);
    car.camera_switch_length = s.switch_length;
    car.camera_switch_clock = s.clock;
    Ok(())
}
