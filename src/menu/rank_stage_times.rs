//! `menu::rank_stage_times` (0x48340B): the four players placed by their stage times (see `menu/results_setups.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Called by `menu::results_482e9b_loop` and `time_trial_results_loop` (set-up, `Step::Call`). Compared in shadow mode. Self-test: `results_setups::test_results`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::results_setups::{dispatch_results, rank_stage_times, test_results};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::rank_stage_times", rva(0x48340B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_results(&HOOK, r, call_original, &rank_stage_times);
}

fn self_test() -> (usize, usize) {
    test_results(&HOOK, call_original, &rank_stage_times)
}
