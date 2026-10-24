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
//! Self-test: a copy of the original (its step count the test's) on a test block, 6
//! fillings.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::rva;
use rc2k_core::drive::camera_views::CameraView;

pub const STEP_RVA: usize = rva(0x719FE0); // dword_719FE0
unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust version on the block at `s`, the step count at `step`.
// Converted: `rc2k_core::drive::camera_views::CameraView::take_wanted`.
pub unsafe fn set_view_at(s: u32, step: u32) {
    unsafe {
        let mut v = CameraView {
            wanted: (s.wrapping_add(0x688) as *const u32).read_unaligned(),
            ..CameraView::default()
        };
        v.take_wanted((step as *const u32).read_unaligned());
        put(s + 0x684, v.view);
        put(s + 0x694, v.held);
        put(s + 0x69C, v.w69c);
        put(s + 0x698, v.w698);
        put(s + 0x690, v.taken_at);
    }
}
