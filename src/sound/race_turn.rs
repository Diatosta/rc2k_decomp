//! `sound::race_turn`: the race's sounds, once a frame.
//!
//! Quitting (the game closing): the program is ended (`standalone::leave_on_close`). Else, once the
//! sounds are started (`RaceClock.started` 1): `sound::step_race_clock`; with a turn due
//! `sound::mark_car_voices`, `car_sounds_all`, `play_cue_queues`, `stage_sounds`, `spot_sounds` and
//! `send_end_of_turn`; and either way `input::step_effects_clock`. Refuses what its callees refuse.

use crate::errors::Refused;

/// The race's sounds for the frame.
pub fn race_turn(
    o: crate::owners::Owners<'static>,
    race: &crate::race::owner::Race,
) -> Result<(), Refused> {
    let (sound, input, audio, system) = (o.sound, o.input, o.audio, o.system);
    if system.session.flags.quitting.is_set() {
        // The program ends here, as the game's start ends it.
        crate::standalone::leave_on_close(o);
    }
    if sound.race.clock.get().started.get() != 1 {
        return Ok(());
    }
    if crate::sound::turn_parts::step_clock(sound, &system.session.picture_clock) {
        crate::platform::timing::note_sound_turn();
        use crate::sound::*;
        mark_car_voices::mark_car_voices(sound, race)?;
        car_sounds_all::car_sounds_all(sound, race, system, audio)?;
        play_cue_queues::play_cue_queues(sound, audio)?;
        stage_sounds::stage_sounds(sound, audio, system, &race.shared.random)?;
        spot_sounds::spot_sounds(sound, audio, system, &race.shared.random)?;
        send_end_of_turn::send_end_of_turn(audio);
    }
    crate::input::step_effects_clock::step_effects_clock(input, sound, race, system)
}
