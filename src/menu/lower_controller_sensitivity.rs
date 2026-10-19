//! `menu::lower_controller_sensitivity` (0x46B414): the controller's sensitivity one less (not below 0), set up (see `menu/controller_items.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! The an item action of the controller pages. Shadow mode compares the two on the player's record, the device record
//! and the command. Self-test: `controller_items::test_item`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::controller_items::{dispatch_item, lower_controller_sensitivity, test_item};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::lower_controller_sensitivity",
    rva(0x46B414),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { lower_controller_sensitivity() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_item(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    test_item(&HOOK, call_original, &rust)
}
