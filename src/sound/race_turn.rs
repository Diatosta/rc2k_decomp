//! `sound::race_turn` (0x40D5F3): the race's sounds, once a frame.
//!
//! Quitting (`g_quitting` 1, the game closing): the program is ended as `start` ends it
//! (`standalone::leave_on_close`); the original jumped out of the loop (chunk 0x4154B5).
//! Else, once the sounds are
//! started (`RaceClock.started` 1): `sound::step_race_clock`; with a turn due (carry clear)
//! `sound::mark_car_voices`, `car_sounds_all`, `play_cue_queues`, `stage_sounds`,
//! `spot_sounds`, `send_end_of_turn`; and either way `input::step_effects_clock`.
//! One caller, 0x421BCF (`sound::race_turn_if`).
//! Refuses what its callees refuse.

use crate::errors::Refused;
use crate::system::owner::system;

/// The race's sounds for the frame.
pub fn race_turn(
    sound: &crate::sound::owner::Sound,
    input: &crate::input::owner::Input,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    if system().session.flags.quitting.is_set() {
        // The original's quit jump (into `system::main_loop`'s way out): the program
        // ended as `start` ends it.
        crate::standalone::leave_on_close();
    }
    if sound.race.clock.started.get() != 1 {
        return Ok(());
    }
    let _step = crate::write_watch::step("sound_turn");
    if crate::sound::turn_parts::step_clock(sound) {
        crate::platform_timing::note_sound_turn();
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
