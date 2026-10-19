//! `menu::score_two_players` (0x485B0E): the two players' places, points, bonuses and totals (see `menu/results_setups.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Called by `menu::results_48584b_loop` (set-up). Compared in shadow mode. Self-test: `results_setups::test_results`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::results_setups::{dispatch_results, score_two_players, test_results};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::score_two_players", rva(0x485B0E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_results(&HOOK, r, call_original, &score_two_players);
}

fn self_test() -> (usize, usize) {
    test_results(&HOOK, call_original, &score_two_players)
}
