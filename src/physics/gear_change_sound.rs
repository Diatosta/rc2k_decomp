//! Tells the current car's sound object of the car's `gear_changed`:
//! at 1 two of its flags are set to 1 (the `drive_slip_speed` side and the car flags); at -1
//! the `velocity` side flag and the car flags are set to 1. A reading: a gear change up or down
//! heard.
//!
//! Returns the sound object.
//!
//! Refuses (before anything) a null sound object when it would be written.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use rc2k_core::physics::gearbox::{GearChange, gear_change};

/// On the current car's sound object and the word at this place.
// Core: `rc2k_core::physics::gearbox::gear_change`.
fn gear_change_sound_at(
    o: &crate::sound::store::SoundObjectCell,
    word: i32,
) -> Result<(), Refused> {
    if let Some(change) = gear_change(word) {
        match change {
            GearChange::Up => o.borrow_mut().slots[2].start = 1,
            GearChange::Down => o.borrow_mut().slots[3].start = 1,
        }
        o.borrow_mut().new_revs = 1;
    }
    Ok(())
}

pub(super) fn gear_change_sound(
    car: &Car,
    sound: &crate::sound::owner::Sound,
    work: &crate::race::work_store::RaceWork,
) -> Result<(), Refused> {
    gear_change_sound_at(
        crate::sound::store::current_object(sound, work),
        car.gear_changed,
    )
}
