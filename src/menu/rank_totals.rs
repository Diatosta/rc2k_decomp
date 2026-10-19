//! `menu::rank_totals` (0x48356F): the four players placed by their totals (see `menu/results_setups.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Called by `menu::screen_483163_loop` (set-up). Compared in shadow mode. Self-test: `results_setups::test_results`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::results_setups::{dispatch_results, rank_totals, test_results};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::rank_totals", rva(0x48356F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_results(&HOOK, r, call_original, &rank_totals);
}

fn self_test() -> (usize, usize) {
    test_results(&HOOK, call_original, &rank_totals)
}
