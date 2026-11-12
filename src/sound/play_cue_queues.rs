//! `sound::play_cue_queues` (0x40ED13): the spoken cues of the two players' cars.
//!
//! Unless no sound object is in use (`g_raceSoundObjectCount` 0): `sound::play_cue_queue`
//! for the object of car `RaceSounds.player_cars[0]`, then unless `RaceSounds.one_player` is 1 (one player) for
//! that of `RaceSounds.player_cars[1]`. One caller, `sound::race_turn`.
//! Refuses (before anything) a car past 7 or a null object; what the queues refuse puts
//! back what was written (a sound already sent stays sent).

use crate::errors::{BadInput, Refused};

/// The players' cars' cue queues played.
pub fn play_cue_queues(
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    if sound.race.sounds.count.get() == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::sound::turn::cue_cars`.
    let cars = rc2k_core::sound::turn::cue_cars(
        sound.race.sounds.player_cars[0].get(),
        || sound.race.sounds.player_cars[1].get(),
        sound.race.sounds.one_player.get(),
    );
    if cars.iter().any(|&n| n > 7) {
        return Err(BadInput("a player's car past 7").into());
    }
    for n in cars {
        crate::sound::play_cue_queue::play_cue_queue(
            crate::sound::store::SoundObject::number(n)?,
            sound,
            audio,
        )?;
    }
    Ok(())
}
