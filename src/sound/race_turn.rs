//! `sound::race_turn` (0x40D5F3): the race's sounds, once a frame.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! Quitting (`g_quitting` 1, the game closing): the original's jump (chunk 0x4154B5:
//! esp = `g_mainLoopStack` and on into `system::main_loop`, or to 0x401116 when that is
//! 0) was run as it is; in the standalone the program
//! is ended as `start` ends it (`standalone::leave_on_close`). Owner, 2026-10-18. Else, once the sounds are
//! started (`dword_5124E4` 1): `sound::step_race_clock`; with a turn due (carry clear)
//! `sound::mark_car_voices`, `car_sounds_all`, `play_cue_queues`, `stage_sounds`,
//! `spot_sounds`, `send_end_of_turn`; and either way `input::step_effects_clock`.
//! One caller, `sub_421BCF` (`sound::race_turn_if`).
//! Refuses what its callees refuse.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::system::owner::system;

// ---- Rust version ----

/// The race's sounds for the frame.
pub fn race_turn(
    sound: &crate::sound::owner::Sound,
    input: &crate::input::owner::Input,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    if system().session.flags.quitting.get() == 1 {
        // The original's quit jump (into `system::main_loop`'s way out): the program
        // ended as `start` ends it.
        crate::standalone::leave_on_close();
    }
    if sound.race.clock.started.get() != 1 {
        return Ok(());
    }
    if crate::sound::turn_parts::step_clock(sound) {
        use crate::sound::*;
        mark_car_voices::mark_car_voices(sound)?;
        car_sounds_all::car_sounds_all(sound)?;
        play_cue_queues::play_cue_queues(sound)?;
        stage_sounds::stage_sounds(sound, audio)?;
        spot_sounds::spot_sounds(sound, audio)?;
        send_end_of_turn::send_end_of_turn();
    }
    crate::input::step_effects_clock::step_effects_clock(input)
}
