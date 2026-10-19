//! `race::edit_camera_2` (0x41E5BA): see `race/edit_camera.rs`: the replay kept for an
//! undo (`race::keep_undo`) and, unless the block shown is the last, `dword_96E480` 2.
//!
//! In:  nothing
//! Out: eax the block shown (`dword_5C2530`); the rest kept
//!
//! Callers: `race::viewer_action` (actions 0x19, 0x1D; no flags read). Shadow mode
//! compares. Self-test: `edit_camera::test_edit`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::race::edit_camera::{edit_camera, edit_regions, test_edit};
use crate::race::viewer::dispatch_regs;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::edit_camera_2", rva(0x41E5BA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(edit_camera(r, 2))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(&HOOK, r, call_original, &edit_regions(), 0, &rust);
}

fn self_test() -> (usize, usize) {
    test_edit(&HOOK, call_original, &rust)
}
