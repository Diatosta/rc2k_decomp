//! `menu::load_welcome_picture` (0x4684BD): the welcome screen's background (see
//! `menu/stage_view_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Called by `menu::welcome_loop` (its background). Shadow mode runs the original alone (it
//! loads a file). Self-test: `stage_view_steps::test_picture`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::stage_view_steps::{dispatch_picture, test_picture};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::load_welcome_picture",
    rva(0x4684BD),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_picture(&HOOK, r, call_original);
}

fn self_test() -> (usize, usize) {
    test_picture(&HOOK)
}
