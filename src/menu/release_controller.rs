//! `menu::release_controller` (0x46B4B4): the input thread asked to give the devices back (see `menu/controller_items.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! The called on leaving the eight controller pages and by the items below. Shadow mode compares the two on the player's record, the device record
//! and the command. Self-test: `controller_items::test_item`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::controller_items::{dispatch_item, release_controller, test_item};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

// The whole function is `call; retn`: the jump is written by hand and the original runs
// from a private copy with its call relocated.
pub static HOOK: RawHook =
    RawHook::new("menu::release_controller", rva(0x46B4B4), thunk, &ORIGINAL)
        .private_copy(6)
        .relocate_calls(&[0])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    release_controller();
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_item(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    test_item(&HOOK, call_original, &rust)
}
