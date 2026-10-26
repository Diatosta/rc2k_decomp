//! `race::edit_camera_1` (0x41E599): see `race/edit_camera.rs`: the replay kept for an
//! undo (`race::keep_undo`) and, unless the block shown is the last, `dword_96E480` 1.
//!
//! In:  nothing
//! Out: eax the block shown (`dword_5C2530`); the rest kept
//!
//! Callers: `race::viewer_action` (actions 0xD, 0x1C, 0x24; no flags read). Shadow mode
//! compares. Self-test: `edit_camera::test_edit`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::race::edit_camera::{edit_camera, edit_regions, test_edit};
use crate::race::viewer::{dispatch_regs, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::edit_camera_1", rva(0x41E599), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(edit_camera(r, 1))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(&HOOK, r, call_original, &edit_regions(), 0, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    test_edit(&HOOK, call_original, &rust)
}
