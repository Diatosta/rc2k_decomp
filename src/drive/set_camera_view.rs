//! `drive::set_camera_view` (0x43A9BA): a block's view taken: +0x684 = the wanted +0x688, +0x694,
//! +0x69C, +0x698 0, +0x690 = the step count `step_clock`. It starts with a `jmp` over a test
//! that nothing reaches (0x43A9BC: the same view kind by `view_kind`, then +0x694 1 and +0x698
//! 40). The block's camera view (+0x684, 0 .. 0x33, drawn by `physics::update_camera`) is
//! switched there; the change-view key (`drive::next_camera_view`, from `read_car_controls`) sets
//! the wanted +0x688 and calls this, `race::race_keys` too.
//!
//! Takes: the block
//!
//! A `jmp` is its first instruction: `.private_copy` with `.manual_patch`. Callers:
//! `drive::next_camera_view`, `race::race_keys`, and code at 0x43B62E.

use crate::errors::BadInput;
use crate::physics::car_store::Car;
use crate::session::Dword;
use rc2k_core::drive::camera_views::CameraView;

/// On the block `car`, the step count `step`.
// Converted: `rc2k_core::drive::camera_views::CameraView::take_wanted`.
pub fn set_view_at(car: &Car, step: &Dword) -> Result<(), BadInput> {
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
