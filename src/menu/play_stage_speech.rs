//! `menu::play_stage_speech` (0x47F437): the stage's speech in the championship (see
//! `menu/stage_view_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Called by `menu::view_stage_loop` and `next_stage_loop` (after the first frame) and the stage pages' steps. Shadow mode runs the original alone (it starts a stream). Self-test: `stage_view_steps::test_speech`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::stage_view_steps::{dispatch_speech, test_speech};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::play_stage_speech", rva(0x47F437), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_speech(&HOOK, r, call_original, false);
}

fn self_test() -> (usize, usize) {
    test_speech(&HOOK, false)
}
