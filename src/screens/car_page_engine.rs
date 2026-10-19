//! `screens::car_page_engine` (0x469F12): the car specification's page 1 (see
//! `screens/car_pages.rs`).
//!
//! In:  edi = the car's record
//! Out: every register kept; the carry of the last text drawn
//! Self-test: `car_pages::test_page`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::screens::car_pages::{dispatch_page, test_page};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::car_page_engine", rva(0x469F12), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_page(&HOOK, r, call_original, 1);
}

fn self_test() -> (usize, usize) {
    test_page(&HOOK, call_original, 1)
}
