//! `sound::play_cue_queues`: the spoken cues of the two players' cars.
//!
//! Unless no sound object is in use: `sound::play_cue_queue` for the object of car
//! `RaceSounds.player_cars[0]`, then unless `RaceSounds.one_player` is 1 for that of
//! `RaceSounds.player_cars[1]`; in a split screen of three or four players, then for the objects
//! 2 and 3 as well (`sound::cue_players`). Refuses (before anything) a car past 7 or a null object; what the
//! queues refuse puts back what was written (a sound already sent stays sent).

use crate::errors::{BadInput, Refused};

/// The players' cars' cue queues played.
pub fn play_cue_queues(
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    if sound.threads.shared.count.get() == 0 {
        return Ok(());
    }
    // The logic is `rc2k_core::sound::turn::cue_cars`.
    let cars = rc2k_core::sound::turn::cue_cars(
        sound.race.sounds().player_cars[0],
        || sound.race.sounds().player_cars[1],
        sound.threads.shared.one_player.get(),
    );
    if cars.iter().any(|&n| n > 7) {
        return Err(BadInput("a player's car past 7").into());
    }
    // A split screen's third and fourth cars speak with objects of their own.
    for n in cars
        .into_iter()
        .chain(2..crate::sound::cue_players::count().min(4))
    {
        crate::sound::play_cue_queue::play_cue_queue(
            crate::sound::store::SoundObject::of(sound, n)?,
            sound,
            audio,
        )?;
    }
    Ok(())
}
