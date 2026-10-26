//! `drive::quit_question` (0x445069): the "quit?" question in a race. The race's sounds
//! stopped, `dword_6235E8` 1; the best whole frame less 7 (the first block's +8 / 100)
//! kept in `dword_70F3E8` when not below it (by the sign); the eight blocks' notice 0xC2
//! (with `dword_6238C0`) or 0xC1, +0x730 0, +0x72C -1. Then until answered
//! (`input::yes_no_key`, the race's frame drawn each turn, `sub_4541BF` through the game):
//! yes -1, no 0, and outside mode 2 with `dword_6238C0` the third answer -2. The notices
//! cleared, `dword_6235E8` 0 and the sounds woken, the eight blocks' +0x468 0.
//!
//! In:  nothing
//! Out: eax the answer; the rest as the steps leave them; no flags read
//!
//! Caller: `drive::race_loop`. It begins with a call: hooked by a private copy of its
//! 0x30D bytes (four calls relocated) and a hand-made patch; ungated (it waits for the
//! player).
//! Refuses what its calls refuse. Shadow mode runs the original alone.
//! Self-test: skipped (it waits for an answer).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::drive::race_parts::{race_frame, stop_race_sounds, wake_car_sounds};
use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::physics::flow::{g, p};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use rc2k_core::drive::race_ticks;

const B: usize = 0x70F3EC;
const NEXT: usize = 0xFE0;

/// The Rust version.
// Converted: `rc2k_core::drive::race_ticks::{best_frame, keeps_frame, quit_notice,
// quit_answer}`.
pub unsafe fn quit_question(entry: &Regs) -> Result<Left, Refused> {
    let mut r = *entry;
    stop_race_sounds(&mut r)?;
    p(0x6235E8, 1);
    r.ebx = 100;
    (r.eax, r.edx) = race_ticks::best_frame(g(B + 8));
    if race_ticks::keeps_frame(r.eax, g(0x70F3E8)) {
        p(0x70F3E8, r.eax);
    }
    let kind = race_ticks::quit_notice(g(0x6238C0) != 0);
    for c in 0..8 {
        p(B + NEXT * c + 0x71C, kind);
    }
    for c in 0..8 {
        p(B + NEXT * c + 0x730, 0);
    }
    for c in 0..8 {
        p(B + NEXT * c + 0x72C, u32::MAX);
    }
    let answer = loop {
        r.eax = unsafe { crate::input::yes_no_key::yes_no_key(r.eax) }?;
        match race_ticks::quit_answer(r.eax, g(0x62389C), g(0x6238C0) != 0) {
            Some(answer) => break answer,
            None => race_frame(&mut r),
        }
    };
    r.eax = answer;
    for field in [0x728, 0x71C, 0x724] {
        for c in 0..8 {
            p(B + NEXT * c + field, 0);
        }
    }
    p(0x6235E8, 0);
    wake_car_sounds(&r)?;
    for c in 0..8 {
        p(B + NEXT * c + 0x468, 0);
    }
    Ok(left_of(&r).with(EAX, answer))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { quit_question(r) }?;
    left.apply(r);
    Ok(())
}
