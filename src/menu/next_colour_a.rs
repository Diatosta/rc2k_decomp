//! `menu::next_colour_a` (0x471AE4): `dword_7554C0` one more (6 to 0) and its pick (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! A left/right handler of an options page's item. Compared in shadow mode. Self-test:
//! `option_steps::test_plain`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::option_steps::{COLOUR_A, dispatch_plain, step_colour, test_plain};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::next_colour_a", rva(0x471AE4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    step_colour(COLOUR_A, true);
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_plain(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    test_plain(&HOOK, call_original, &rust)
}
