//! `menu::arcade_level_4` (0x487D9E): the arcade start menu's level 4 item (see
//! `menu/arcade_levels.rs`).
//!
//! In:  nothing
//! Out: eax -1 for a closed level; else as the event leaves them
//! Ungated thunk; shadow mode runs the original alone. Self-test: `arcade_levels::test_level`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::arcade_levels::{dispatch_level, test_level};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::arcade_level_4", rva(0x487D9E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_level(&HOOK, r, call_original, 4);
}

fn self_test() -> (usize, usize) {
    test_level(&HOOK, 0x487D9E, 4)
}
