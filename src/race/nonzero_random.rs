//! `race::nonzero_random` (0x444C14): `math::next_random`, 1 in place of 0.
//!
//! In:  nothing
//! Out: eax = the number; the rest preserved (the flags are `next_random`'s or `inc`'s;
//!      the one caller does not read them)
//!
//! See `race/race_init.rs`. One caller, `race::seed_car_randoms` (0x444BF6). It begins
//! with its call: private copy, the call relocated, manual patch. Shadow mode: the Rust
//! version, the generator's state put back, the original's, compared. Self-test: the
//! states of `math::next_random`'s test (0 among them, whose next number is 0).

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::math::next_random::{STATE_RVA, TEST_STATES};
use crate::race::race_init::nonzero_random;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::nonzero_random", rva(0x444C14), thunk, &ORIGINAL)
    .private_copy(0x9)
    .relocate_calls(&[0x00])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { nonzero_random() }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(STATE_RVA)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [at(STATE_RVA)];
    let saved = capture(&regions);
    for state in TEST_STATES {
        check_state(
            &mut t,
            &format!("state={state:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(STATE_RVA, state) },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
