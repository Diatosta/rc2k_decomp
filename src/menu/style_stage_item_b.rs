//! `menu::style_stage_item_b` (0x470FC3): menu item 1's look for the stage view (see
//! `menu/stage_view_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Called by `menu::view_stage_loop`. Compared in shadow mode. Self-test:
//! `stage_view_steps::test_style`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::stage_view_steps::{dispatch_plain, style_stage_item, test_style};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::style_stage_item_b", rva(0x470FC3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_plain(&HOOK, r, call_original, &|_| style_stage_item(true));
}

fn self_test() -> (usize, usize) {
    test_style(&HOOK, call_original, true)
}
