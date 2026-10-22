//! `menu::choose_key_8` (0x46837B): start redefining the key at place 8.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/choose_key.rs`. Compared in shadow mode. Self-test: three settings.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::choose_key::{check_choose, choose_entry, dispatch_choose};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::choose_key_8", rva(0x46837B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_choose(&HOOK, r, call_original, 8);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    choose_entry(r, 8)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_choose(&mut t, call_original, 8);
    t.finish()
}
