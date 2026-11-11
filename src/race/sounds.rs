//! The race's sound objects woken at the loading: `RaceSounds.table[0]`, eight
//! pointers to the cars' sound objects (`sound::load_race_sounds`), each with a flag at +8.

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;

const OBJECTS: u32 = 8;
const MOST_CARS: u32 = 0x1_0000;

fn wake(n: u32) -> Result<(), BadInput> {
    crate::sound::store::SoundObject::number(n & 7)?
        .in_use
        .set(1);
    Ok(())
}

/// `race::start_race_sounds` (0x40D592): `sound::start_race_clock`, and unless `RaceSounds.count` is 0
/// it is held to 1 .. 8 and that many objects woken; then `sound::let_sounds_play` and
/// `RaceClock.started` 1.
pub fn start_race_sounds(sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    crate::sound::race_clock::start_race_clock(crate::system::tick_count::tick_count(), sound);
    let count: u32 = sound.race.sounds.count.get();
    if count != 0 {
        let count = count.clamp(1, OBJECTS);
        sound.race.sounds.count.set(count);
        for n in 0..count {
            wake(n)?;
        }
    }
    crate::sound::let_sounds_play::let_sounds_play();
    sound.race.clock.started.set(1);
    Ok(())
}

/// `race::wake_car_sounds` (0x45D255): nothing while `in_credits` is set; else each car's object
/// (`race_cars` of them, the table taken round its eight) woken, then `race::start_race_sounds`.
/// Refuses 0x10000 cars or more (the original counts on to them).
pub fn wake_car_sounds(race: &Race) -> Result<(), Refused> {
    let sound = crate::sound::owner::sound();
    let shared = &race.shared;
    if shared.work.in_credits.is_set() {
        return Ok(());
    }
    let cars: u32 = shared.view.race_cars.get();
    if cars >= MOST_CARS {
        return Err(BadInput("0x10000 cars or more").into());
    }
    for n in 0..cars {
        wake(n)?;
    }
    start_race_sounds(sound)
}
