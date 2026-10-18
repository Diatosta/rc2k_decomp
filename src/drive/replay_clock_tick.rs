//! `drive::replay_clock_tick` (0x460C20): the replay's clock: `dword_5C2534` + 1, at 100
//! back to 0 with the frame `dword_5C2530` + 1.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::replay_step`.
//! Self-test: the original and the Rust version below, at and past 100.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::replay_clock_tick", rva(0x460C20), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub fn replay_clock_tick() {
    p(0x5C2534, g(0x5C2534).wrapping_add(1));
    if g(0x5C2534) >= 0x64 {
        p(0x5C2534, 0);
        p(0x5C2530, g(0x5C2530).wrapping_add(1));
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    replay_clock_tick();
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(a(0x5C2530), 8)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all: Vec<Region> = vec![(a(0x5C2530), 8)];
    let saved = capture(&all);
    for case in 0..5u32 {
        let set = || {
            p(0x5C2534, [0u32, 0x62, 0x63, 0x64, 0x200][case as usize]);
            p(0x5C2530, 7);
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
