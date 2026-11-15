//! Tells the current car's sound object of the car's `gear_changed`:
//! at 1 two of its flags are set to 1 (the `drive_slip_speed` side and the car flags); at -1
//! the `velocity` side flag and the car flags are set to 1. A reading: a gear change up or down
//! heard.
//!
//! Returns the sound object.
//!
//! Caller: `physics::engine_sound`. Refuses (before anything) a null sound object when it
//! would be written.

use crate::errors::Refused;
use crate::memory::session::PlainInt;
use crate::physics::car_store::Car;
use rc2k_core::physics::gearbox::{GearChange, gear_change};

/// On the current car's sound object and the word at this place.
// Core: `rc2k_core::physics::gearbox::gear_change`.
pub fn gear_change_sound_at(
    o: &crate::sound::store::SoundObject,
    word: &PlainInt,
) -> Result<(), Refused> {
    if let Some(change) = gear_change(word.get()) {
        match change {
            GearChange::Up => o.slots[2].start.set(1),
            GearChange::Down => o.slots[3].start.set(1),
        }
        o.new_revs.set(1);
    }
    Ok(())
}

pub fn gear_change_sound(
    car: &Car,
    sound: &crate::sound::owner::Sound,
    work: &crate::race::work_store::RaceWork,
) -> Result<(), Refused> {
    gear_change_sound_at(
        crate::sound::store::current_object(sound, work),
        &car.gear_changed,
    )
}
