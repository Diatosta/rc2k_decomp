//! `drive::hold_count_tick` (0x460C10): `dword_6235EC` counted down to 0 (a reading: a
//! hold after the start).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::replay_step`.
//! Self-test: the original and the Rust version at 0, 1 and more.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::hold_count_tick", rva(0x460C10), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub fn hold_count_tick() {
    if g(0x6235EC) != 0 {
        p(0x6235EC, g(0x6235EC) - 1);
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    hold_count_tick();
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(a(0x6235EC), 4)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all: Vec<Region> = vec![(a(0x6235EC), 4)];
    let saved = capture(&all);
    for case in 0..4u32 {
        let set = || {
            p(0x6235EC, [0u32, 1, 2, u32::MAX][case as usize]);
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
