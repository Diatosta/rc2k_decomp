//! `menu::next_stage_rows` (0x4A375F): the event page's rows on by ten (see `menu/stage_rows.rs`).
//!
//! In:  nothing
//! Out: eax as `stage_rows` says; the rest kept
//! Compared in shadow mode. Self-test: `stage_rows::test_rows`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::stage_rows::{dispatch_rows, test_rows};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::next_stage_rows", rva(0x4A375F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_rows(&HOOK, r, call_original, false, true);
}

fn self_test() -> (usize, usize) {
    test_rows(&HOOK, call_original, false, true)
}
