//! `menu::flip_yes_no` (0x46FA52): the "leave?" question's button turned to the other
//! answer.
//!
//! In:  nothing
//! Out: every register preserved; the flags of the `xor`
//!
//! `g_yesNoAnswer` (0x75524C) `xor` 1. The left and right action of the question's
//! button (the menu table at 0x7544D8).
//! Self-test: four answers.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::flip_yes_no", rva(0x46FA52), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer

// ---- Rust version ----

pub fn flip_yes_no() -> u32 {
    unsafe {
        let answer = game_read::<u32>(ANSWER_RVA) ^ 1;
        game_write(ANSWER_RVA, answer);
        answer
    }
}

// ---- hook ----

fn regions() -> [Region; 1] {
    [at(ANSWER_RVA)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    flip_yes_no();
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved = capture(&regions());
    for answer in [0u32, 1, 2, 0xFFFF_FFFF] {
        check_state(
            &mut t,
            &format!("answer={answer:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(ANSWER_RVA, answer) },
            &regions(),
            &rust,
        );
    }
    restore(&regions(), &saved);
    t.finish()
}
