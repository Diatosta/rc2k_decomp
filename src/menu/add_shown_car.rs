//! `menu::add_shown_car` (0x486BD2): the shown car put at the car list's end (see `menu/arcade_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Compared in shadow mode. Self-test: `arcade_steps::test_step`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::arcade_steps::{add_shown_car, dispatch_step, test_step};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::add_shown_car", rva(0x486BD2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    add_shown_car()?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_step(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    test_step(&HOOK, call_original, &rust, &|_| Regs::SENTINEL)
}
