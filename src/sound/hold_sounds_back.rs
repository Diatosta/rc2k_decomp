//! `sound::hold_sounds_back` (0x409B6B): `g_soundsHeldBack` 1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Its twin `sound::let_sounds_play` (0x409B76) sets it to 0; `sound::play_sound` plays
//! nothing while it is 1. One caller (0x40D388).
//! Self-test: the variable from 1 and from 0, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use crate::sound::let_sounds_play::HELD_BACK_RVA;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::hold_sounds_back", rva(0x409B6B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn hold_sounds_back() {
    unsafe { game_write(HELD_BACK_RVA, 1u32) };
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    hold_sounds_back();
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(HELD_BACK_RVA)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [at(HELD_BACK_RVA)];
    let saved = capture(&regions);
    for from in [0u32, 1] {
        check_state(
            &mut t,
            &format!("from {from}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(HELD_BACK_RVA, from) },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
