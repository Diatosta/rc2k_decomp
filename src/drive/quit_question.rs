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
//! Caller: `drive::race_loop`. Refuses what its calls refuse.

use crate::drive::race_parts::{stop_race_sounds, wake_car_sounds};
use crate::errors::Refused;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::race;
use crate::race::vars_store::RACE_VARS;
use rc2k_core::drive::race_ticks;

/// The Rust version.
// Converted: `rc2k_core::drive::race_ticks::{best_frame, keeps_frame, quit_notice,
// quit_answer}`.
/// The answer (eax): 0 to race on. The other registers it leaves are read by no caller
/// (liveness: `race_loop` reads eax only).
pub fn quit_question() -> Result<u32, Refused> {
    let replay = &race().replay;
    stop_race_sounds()?;
    // A button from before the question answers nothing.
    crate::input::menu_pad::take_answer();
    RACE_VARS.scene_switch.set(1);
    // eax: the best frame, then each key's answer, then what the frame leaves; it goes
    // into `input::yes_no_key` (the key given, when one is).
    let (mut eax, _) = race_ticks::best_frame(CAR_BLOCKS.cars[0].step_clock.bits());
    if race_ticks::keeps_frame(eax, replay.replay_frames.last_kept.get()) {
        replay.replay_frames.last_kept.set(eax);
    }
    let kind = race_ticks::quit_notice(RACE_VARS.no_replay.get() != 0);
    for c in 0..8 {
        CAR_BLOCKS.cars[c as usize].notice_id.set_bits(kind);
    }
    for c in 0..8 {
        CAR_BLOCKS.cars[c as usize].notice_time_b.set_bits(0);
    }
    for c in 0..8 {
        CAR_BLOCKS.cars[c as usize].notice_time.set_bits(u32::MAX);
    }
    let answer = loop {
        // A gamepad's button answers directly (not the original's): this question has no
        // items on screen to choose from.
        eax = match crate::input::menu_pad::take_answer() {
            Some(answer) => answer,
            None => { crate::input::yes_no_key::yes_no_key(eax) }?,
        };
        match race_ticks::quit_answer(
            eax,
            RACE_VARS.run_kind.get(),
            RACE_VARS.no_replay.get() != 0,
        ) {
            Some(answer) => break answer,
            None => eax = crate::drive::race_frame::race_frame(eax)?,
        }
    };
    for c in 0..8 {
        let car = &CAR_BLOCKS.cars[c];
        for cell in [&car.notice_shown, &car.notice_id, &car.notice_last] {
            cell.set(0);
        }
    }
    RACE_VARS.scene_switch.set(0);
    wake_car_sounds()?;
    for c in 0..8 {
        CAR_BLOCKS.cars[c as usize].input_keys.set_bits(0);
    }
    Ok(answer)
}
