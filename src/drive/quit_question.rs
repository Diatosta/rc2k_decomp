//! `drive::quit_question` (0x445069): the "quit?" question in a race. The race's sounds stopped,
//! `scene_switch` 1; the best whole frame less 7 (the first block's +8 / 100) kept in
//! `last_kept` when not below it (by the sign); the eight blocks' notice 0xC2 (with
//! `no_replay`) or 0xC1, +0x730 0, +0x72C -1. Then until answered (`input::yes_no_key`, the
//! race's frame drawn each turn, `drive::race_frame` through the game): yes -1, no 0, and outside
//! mode 2 with `no_replay` the third answer -2. The notices cleared, `scene_switch` 0 and the
//! sounds woken, the eight blocks' +0x468 0.
//!
//! Returns: the answer
//!
//! Caller: `drive::race_loop`. Refuses what its calls refuse.

use crate::drive::race_parts::{stop_race_sounds, wake_car_sounds};
use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::drive::race_ticks;

// Converted: `rc2k_core::drive::race_ticks::{best_frame, keeps_frame, quit_notice,
// quit_answer}`.
/// The answer: 0 to race on.
pub fn quit_question(race: &Race, o: crate::owners::Owners<'static>) -> Result<u32, Refused> {
    let input = o.input;
    let shared = &race.shared;
    let replay = &race.replay;
    stop_race_sounds(o)?;
    // A button from before the question answers nothing.
    crate::input::menu_pad::take_answer();
    shared.vars.get().scene_switch.set(1);
    // The carried value: the best frame, then each key's answer, then what the frame leaves; it goes
    // into `input::yes_no_key` (the key given, when one is).
    let (mut eax, _) = race_ticks::best_frame(race.car_blocks().cars[0].step_clock.get());
    if race_ticks::keeps_frame(eax, replay.replay_frames().last_kept.get()) {
        replay.replay_frames().last_kept.set(eax);
    }
    let kind = race_ticks::quit_notice(shared.vars.get().no_replay.get() != 0);
    for c in 0..8 {
        race.car_blocks().cars[c as usize].notice_id.set(kind);
    }
    for c in 0..8 {
        race.car_blocks().cars[c as usize].notice_time_b.set(0);
    }
    for c in 0..8 {
        race.car_blocks().cars[c as usize].notice_time.set(u32::MAX);
    }
    let answer = loop {
        // A gamepad's button answers directly (not the original's): this question has no
        // items on screen to choose from.
        eax = match crate::input::menu_pad::take_answer() {
            Some(answer) => answer,
            None => {
                crate::input::yes_no_key::yes_no_key(
                    crate::strings::append_string_by_id::StringOwners::of(o),
                    eax,
                    input,
                )
            }?,
        };
        match race_ticks::quit_answer(
            eax,
            shared.vars.get().run_kind.get(),
            shared.vars.get().no_replay.get() != 0,
        ) {
            Some(answer) => break answer,
            None => eax = crate::drive::race_frame::race_frame(race, o, eax)?,
        }
    };
    for c in 0..8 {
        let car = &race.car_blocks().cars[c];
        for cell in [&car.notice_shown, &car.notice_id, &car.notice_last] {
            cell.set(0);
        }
    }
    shared.vars.get().scene_switch.set(0);
    wake_car_sounds(race, o.sound, o.audio)?;
    for c in 0..8 {
        race.car_blocks().cars[c as usize].input_keys.set(0);
    }
    Ok(answer)
}
