//! A block's view taken: +0x684 = the wanted +0x688, +0x694, +0x69C, +0x698 0, +0x690 = the
//! step count `step_clock`. The block's camera view (+0x684, 0 .. 0x33, drawn by
//! `physics::update_camera`) is switched there; the change-view key (`drive::next_camera_view`,
//! from `read_car_controls`) sets the wanted +0x688 and calls this, `race::race_keys` too.

use crate::errors::BadInput;
use crate::physics::car_store::Car;
use crate::session::PlainDword;
use rc2k_core::drive::camera_views::CameraView;

pub fn set_view_at(car: &Car, step: &PlainDword) -> Result<(), BadInput> {
    let mut v = CameraView {
        wanted: car.camera_blend_view.get(),
        ..CameraView::default()
    };
    v.take_wanted(step.get());
    car.camera_view.set(v.view);
    car.camera_switch_active.set(v.held);
    car.camera_switch_share.set(v.switch_share);
    car.camera_switch_length.set(v.w698);
    car.camera_switch_clock.set(v.taken_at);
    Ok(())
}
