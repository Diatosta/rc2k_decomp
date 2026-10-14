//! `menu::choose_key_4` (0x468327): start redefining the key at place 4.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/choose_key.rs`. Compared in shadow mode. Self-test: three settings.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::choose_key::{check_choose, dispatch_choose};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::choose_key_4", rva(0x468327), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_choose(&HOOK, r, call_original, 4);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_choose(&mut t, call_original, 4);
    t.finish()
}
