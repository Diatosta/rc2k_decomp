//! `menu::next_screen_mode` (0x46FFEF): the screen mode chosen one more (after the last, the first) (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: eax `dword_755648`, ebx the last mode's number
//! A left/right handler of an options page's item. Compared in shadow mode. Self-test:
//! `option_steps::test_plain`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::option_steps::{dispatch_plain, next_screen_mode, test_plain};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::next_screen_mode", rva(0x46FFEF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(next_screen_mode())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_plain(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    test_plain(&HOOK, call_original, &rust)
}
