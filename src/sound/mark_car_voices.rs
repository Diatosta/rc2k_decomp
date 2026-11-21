//! `sound::mark_car_voices`: each car's sound object told whether the car sounds and whose voice it
//! has.
//!
//! Only while `RaceSounds.mark_car_voices` is 1: for each of the `RaceSounds.count` objects in use,
//! `in_use` is set to 0; when its car's block has the voice field not 0, `in_use` is 1 and
//! `sound_offset` 0, and unless the field is 1, `sound_offset` is !n (a negative number: a
//! computer's car, by the callers' reading). Refuses (before writing) a count of 0 or over 8, and a
//! null object or car.

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::sound::store::{SoundObject, SoundObjectCell};

/// The (object, car) pairs in use, or a refusal.
fn pairs<'a>(
    sound: &'a crate::sound::owner::Sound,
    race: &Race,
) -> Result<Vec<(&'a SoundObjectCell, usize)>, BadInput> {
    let count = sound.threads.shared.count.get();
    if count == 0 || count > 8 {
        return Err(BadInput("a sound object count of 0 or over 8"));
    }
    let pairs = (0..count as usize)
        .map(|n| Ok((SoundObject::of(sound, n as u32)?, n)))
        .collect::<Result<Vec<_>, BadInput>>()?;
    if pairs
        .iter()
        .any(|&(_, c)| !race.shared.work.get().has_car_block(c))
    {
        return Err(BadInput("a null car"));
    }
    Ok(pairs)
}

/// The logic is `rc2k_core::sound::cars::voice_mark`.
pub fn mark_car_voices(sound: &crate::sound::owner::Sound, race: &Race) -> Result<(), Refused> {
    if sound.race.sounds().mark_car_voices != 1 {
        return Ok(());
    }
    for (n, (o, car)) in pairs(sound, race)?.into_iter().enumerate() {
        let kind = race
            .frame
            .cars
            .car(race.shared.work.get().block_number(car)?)?
            .driver_kind
            .get();
        let (sounds, voice) = rc2k_core::sound::cars::voice_mark(n as u32, kind);
        o.borrow_mut().in_use = sounds;
        if let Some(voice) = voice {
            o.borrow_mut().sound_offset = voice;
        }
    }
    Ok(())
}
