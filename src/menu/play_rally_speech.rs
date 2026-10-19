//! `menu::play_rally_speech` (0x47F40F): the rally's speech in the championship (see
//! `menu/stage_view_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Called by `menu::welcome_loop` (after the first frame). Shadow mode runs the original alone (it starts a stream). Manual
//! patch with a private copy (a call at the second byte). Self-test: `stage_view_steps::test_speech`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::stage_view_steps::{dispatch_speech, test_speech};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::play_rally_speech", rva(0x47F40F), thunk, &ORIGINAL)
    .private_copy(0x28)
    .relocate_calls(&[0x1, 0x21])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_speech(&HOOK, r, call_original, true);
}

fn self_test() -> (usize, usize) {
    test_speech(&HOOK, true)
}
