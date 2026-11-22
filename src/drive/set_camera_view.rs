//! A block's view taken: `camera_view` = the wanted `camera_blend_view`, `camera_switch_active`,
//! `camera_switch_share`, `camera_switch_length` 0, `camera_switch_clock` = the step count
//! `step_clock`. The block's camera view (`camera_view`, 0 .. 0x33, drawn by
//! `physics::update_camera`) is switched there; the change-view key (`drive::next_camera_view`,
//! from `read_car_controls`) sets the wanted `camera_blend_view` and calls this, `race::race_keys` too.

use crate::errors::BadInput;
use crate::physics::car_store::Car;
use rc2k_core::drive::camera_views::CameraView;

pub(super) fn set_view_at(car: &mut Car, step: u32) -> Result<(), BadInput> {
    let mut v = CameraView {
        wanted: car.camera_blend_view,
        ..CameraView::default()
    };
    v.take_wanted(step);
    car.camera_view = v.view;
    car.camera_switch_active = v.held;
    car.camera_switch_share = v.switch_share;
    car.camera_switch_length = v.w698;
    car.camera_switch_clock = v.taken_at;
    Ok(())
}
