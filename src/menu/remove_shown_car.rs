//! `menu::remove_shown_car` (0x486BF5): the shown car taken out of its list, the numbers listed again (see `menu/arcade_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Compared in shadow mode. Self-test: `arcade_steps::test_step`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::arcade_steps::{dispatch_step, remove_shown_car, test_step};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::remove_shown_car", rva(0x486BF5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    remove_shown_car(&mut || unsafe {
        crate::saves::list_car_numbers::list_car_numbers().map_err(Refused::from)
    })?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_step(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    test_step(&HOOK, call_original, &rust, &|_| Regs::SENTINEL)
}
