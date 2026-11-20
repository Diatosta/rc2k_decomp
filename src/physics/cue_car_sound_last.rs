//! `physics::cue_car_sound`'s twin for the pace notes: the cue goes last in the queue of the
//! sound object of a player's car (0 or 1; up to 3 in a split screen of more), when the object's state word is 1.
//!
//! Refuses (before anything) a car past 7 or a null sound object.

use crate::errors::Refused;
use crate::sound::store::SoundObject;
use rc2k_core::physics::listed_cars::cue_object;

/// `cue` queued last for the car `which` (a player's), if it sounds.
// Core: `rc2k_core::physics::listed_cars::cue_object`.
pub(super) fn cue_car_sound_last(
    which: u32,
    cue: u32,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    let failed = std::cell::Cell::new(None);
    let found = cue_object(
        which,
        crate::sound::cue_players::count(),
        |which| match which {
            // The split screen's third and fourth cars are their own sound objects.
            0 | 1 => sound.race.sounds.get().player_cars[which as usize].get(),
            other => other,
        },
        |car| car,
        |car| match SoundObject::of(sound, car) {
            Ok(o) => o.in_use.get(),
            Err(e) => {
                failed.set(Some(e));
                0
            }
        },
    )?;
    if let Some(e) = failed.take() {
        return Err(e.into());
    }
    let Some(car) = found else {
        return Ok(());
    };
    crate::physics::queue_sound_cue::queue_sound_cue(
        false,
        cue,
        SoundObject::of(sound, car)?,
        audio,
        sound.race.cue_lists.get(),
    )
}
