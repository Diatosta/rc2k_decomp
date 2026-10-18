//! `math::stir_random` (0x40188B): stirs a number into the random numbers' state.
//!
//! In:  eax = the number
//! Out: every register preserved
//! One step of the generator on eax with the state mixed in; unless that is 0, it becomes
//! the state. One caller, `start`, after `sub_403131` (which asks about the volume and
//! ends with a `pop eax` that takes the esi it had saved: so what is stirred in is
//! whatever esi was at the process's start, not the volume's number).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::math::next_random::{STATE_RVA, TEST_STATES, random_step};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::stir_random", rva(0x40188B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn stir_random(number: u32) {
    crate::math::next_random::wait_for_random();
    unsafe {
        let stirred = random_step(number, game_read(STATE_RVA));
        if stirred != 0 {
            game_write(STATE_RVA, stirred);
        }
    }
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { stir_random(r.eax) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    // Waits while a test owns the generator (`next_random::own_random`).
    crate::math::next_random::wait_for_random();
    dispatch_state(&HOOK, r, call_original, &[at(STATE_RVA)], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [at(STATE_RVA)];
    let saved = capture(&regions);
    for state in TEST_STATES {
        for number in TEST_STATES.into_iter().chain([state.rotate_left(1), state]) {
            check_state(
                &mut t,
                &format!("state={state:#x} number={number:#x}"),
                call_original,
                Regs {
                    eax: number,
                    ..Regs::SENTINEL
                },
                &|| unsafe { game_write(STATE_RVA, state) },
                &regions,
                &rust,
            );
        }
    }
    restore(&regions, &saved);
    t.finish()
}
