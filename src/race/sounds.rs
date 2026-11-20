//! Wakes the race's sound objects at the loading: `RaceSounds.table[0]` holds eight cars' sound
//! objects (`sound::load_race_sounds`), each with an in-use flag.

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;

const OBJECTS: u32 = 8;
const MOST_CARS: u32 = 0x1_0000;

fn wake(sound: &crate::sound::owner::Sound, n: u32) -> Result<(), BadInput> {
    crate::sound::store::SoundObject::of(sound, n & 7)?
        .in_use
        .set(1);
    Ok(())
}

/// `race::start_race_sounds`: starts the race clock (`sound::start_race_clock`); unless
/// `RaceSounds.count` is 0 it is held to 1 .. 8 and that many objects are woken. Then the sounds
/// are let play (`sound::let_sounds_play`) and `RaceClock.started` is set to 1.
pub(super) fn start_race_sounds(
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
    picture_clock: &crate::clock::tick_count::PictureClock,
) -> Result<(), Refused> {
    crate::sound::race_clock::start_race_clock(
        crate::clock::tick_count::tick_count(picture_clock),
        sound,
    );
    let count: u32 = sound.threads.shared.count.get();
    if count != 0 {
        let count = count.clamp(1, OBJECTS);
        sound.threads.shared.count.set(count);
        for n in 0..count {
            wake(sound, n)?;
        }
    }
    crate::sound::let_sounds_play::let_sounds_play_in(audio);
    sound.race.clock.get().borrow_mut().started = 1;
    Ok(())
}

/// `race::wake_car_sounds`: does nothing while `in_credits` is set; else wakes each car's object
/// (`race_cars` of them, the table taken round its eight), then runs `race::start_race_sounds`.
/// Refuses 0x10000 cars or more.
pub fn wake_car_sounds(
    race: &Race,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
    picture_clock: &crate::clock::tick_count::PictureClock,
) -> Result<(), Refused> {
    let shared = &race.shared;
    if shared.work.get().in_credits.is_set() {
        return Ok(());
    }
    let cars: u32 = shared.view.get().race_cars.get();
    if cars >= MOST_CARS {
        return Err(BadInput("0x10000 cars or more").into());
    }
    for n in 0..cars {
        wake(sound, n)?;
    }
    start_race_sounds(sound, audio, picture_clock)
}
