//! `menu::raise_dead_zone_b` (0x46B3EA): the second dead zone (+0x44) one more while the two leave more than 10 (see `menu/controller_items.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! The an item action of the controller pages. Shadow mode compares the two on the player's record, the device record
//! and the command. Self-test: `controller_items::test_item`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::controller_items::{dead_zone_left, dispatch_item, test_item};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::raise_dead_zone_b", rva(0x46B3EA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    dead_zone_left(0x44)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_item(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    test_item(&HOOK, call_original, &rust)
}
