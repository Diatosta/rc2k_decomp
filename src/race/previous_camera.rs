//! `race::previous_camera` (0x41E654): see `race/edit_camera.rs`: unless the block shown
//! is the last, the camera edited (`dword_96E480`) of the block shown one back, the
//! skipped cameras passed over, the change spread over the next blocks that had the old
//! one (`race::spread_camera`).
//!
//! In:  nothing
//! Out: the block the last: eax the block. Else as `spread_camera` leaves them (eax, ecx),
//!      ebx the camera byte, dl the new camera, dh the old; no flags read
//!
//! Callers: `race::viewer_action` (actions 0x1E, 0x24). Refused, before anything is
//! written: every camera skipped (the original goes round for ever) and what
//! `spread_camera` refuses. Shadow mode compares. Self-test: `edit_camera::test_step`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::race::edit_camera::{camera_regions, step_camera, test_step};
use crate::race::viewer::{dispatch_regs, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::previous_camera", rva(0x41E654), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Regs, Refused> {
    unsafe { step_camera(r, -1) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(&HOOK, r, call_original, &camera_regions(), 0, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    test_step(&HOOK, call_original, &rust)
}
