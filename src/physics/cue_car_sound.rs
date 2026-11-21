//! For a player's car number (0 or 1; up to 3 in a split screen of more), the sound object of
//! that car: when its state word is 1,
//! `physics::queue_sound_cue` puts the cue first in its queue, using the object's
//! `parts_present`.
//!
//! Refuses (before anything) a car past 7 or a null sound object.

use crate::errors::Refused;
use crate::sound::store::SoundObject;
use rc2k_core::physics::listed_cars::cue_object;

/// `cue` queued first for the car `which` (a player's), if it sounds.
// Core: `rc2k_core::physics::listed_cars::cue_object`.
pub(super) fn cue_car_sound(
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
            0 | 1 => sound.race.sounds().player_cars[which as usize],
            other => other,
        },
        |car| car,
        |car| match SoundObject::of(sound, car) {
            Ok(o) => o.borrow().in_use,
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
        true,
        cue,
        SoundObject::of(sound, car)?,
        audio,
        sound.race.cue_lists.get(),
    )
}
