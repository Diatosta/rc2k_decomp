//! `menu::next_stage_page` (0x46FB27): the stage's next page, its map and speech (see
//! `menu/stage_view_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! A left/right handler of the stage view's item. Shadow mode runs the original alone (it
//! draws the map and starts a stream). Self-test: `stage_view_steps::test_page`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::stage_view_steps::{dispatch_page, test_page};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::next_stage_page", rva(0x46FB27), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_page(&HOOK, r, call_original, true);
}

fn self_test() -> (usize, usize) {
    test_page(&HOOK, 0x46FB27, true)
}
