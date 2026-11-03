//! `drive::set_camera_view` (0x43A9BA): a block's (esi) view taken: +0x684 = the wanted
//! +0x688, +0x694, +0x69C, +0x698 0, +0x690 = the step count `dword_719FE0` (the work
//! copy's +8, whatever esi is). It starts with a `jmp` over a test that nothing reaches
//! (0x43A9BC: the same view kind by `dword_6063F0`, then +0x694 1 and +0x698 40).
//! By the look of it the camera views (`drive::next_camera_view` cycles +0x688; a
//! reading).
//!
//! In:  esi the block
//! Out: eax the step count; the rest kept; no flags read (callers go on with `mov`,
//!      `retn`)
//!
//! A `jmp` is its first instruction: `.private_copy` with `.manual_patch`.
//! Callers: `drive::next_camera_view`, `sub_43A649` (0x43A94F), and code at 0x43B62E.

use crate::errors::BadInput;
use crate::physics::car_store::Car;
use crate::session::Dword;
use rc2k_core::drive::camera_views::CameraView;

/// The Rust version on the block `car`, the step count `step`.
// Converted: `rc2k_core::drive::camera_views::CameraView::take_wanted`.
pub fn set_view_at(car: &Car, step: &Dword) -> Result<(), BadInput> {
    let mut v = CameraView {
        wanted: car.camera_blend_view.get(),
        ..CameraView::default()
    };
    v.take_wanted(step.bits());
    car.camera_view.set(v.view);
    car.camera_switch_active.set(v.held);
    car.camera_switch_share.set_bits(v.w69c);
    car.camera_switch_length.set(v.w698);
    car.camera_switch_clock.set(v.taken_at);
    Ok(())
}
