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

use crate::drive::race_parts::{stop_race_sounds, wake_car_sounds};
use crate::errors::Refused;
use crate::physics::flow::{g, p};
use rc2k_core::drive::race_ticks;

const B: usize = 0x70F3EC;
const NEXT: usize = 0xFE0;

/// The Rust version.
// Converted: `rc2k_core::drive::race_ticks::{best_frame, keeps_frame, quit_notice,
// quit_answer}`.
/// The answer (eax): 0 to race on. The other registers it leaves are read by no caller
/// (liveness: `race_loop` reads eax only).
pub unsafe fn quit_question() -> Result<u32, Refused> {
    stop_race_sounds()?;
    p(0x6235E8, 1);
    // eax: the best frame, then each key's answer, then what the frame leaves; it goes
    // into `input::yes_no_key` (the key given, when one is).
    let (mut eax, _) = race_ticks::best_frame(g(B + 8));
    if race_ticks::keeps_frame(eax, g(0x70F3E8)) {
        p(0x70F3E8, eax);
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
        eax = unsafe { crate::input::yes_no_key::yes_no_key(eax) }?;
        match race_ticks::quit_answer(eax, g(0x62389C), g(0x6238C0) != 0) {
            Some(answer) => break answer,
            None => eax = crate::drive::race_frame::race_frame(eax)?,
        }
    };
    for field in [0x728, 0x71C, 0x724] {
        for c in 0..8 {
            p(B + NEXT * c + field, 0);
        }
    }
    p(0x6235E8, 0);
    wake_car_sounds()?;
    for c in 0..8 {
        p(B + NEXT * c + 0x468, 0);
    }
    Ok(answer)
}
