//! For car number 0 or 1, the sound object of that car: when its state word is 1,
//! `physics::queue_sound_cue` puts the cue first in its queue, using the object's
//! `parts_present`.
//!
//! Refuses (before anything) a car past 7 or a null sound object.

use crate::errors::Refused;
use crate::sound::store::SoundObject;
use rc2k_core::physics::listed_cars::cue_object;

/// `cue` queued first for the car `which` (0 or 1: a player's), if it sounds.
// Core: `rc2k_core::physics::listed_cars::cue_object`.
pub fn cue_car_sound(
    which: u32,
    cue: u32,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    let failed = std::cell::Cell::new(None);
    let found = cue_object(
        which,
        |which| sound.race.sounds.get().player_cars[which as usize].get(),
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
        true,
        cue,
        SoundObject::of(sound, car)?,
        audio,
        sound.race.cue_lists.get(),
    )
}
