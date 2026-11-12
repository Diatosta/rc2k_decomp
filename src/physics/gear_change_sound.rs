//! `physics::gear_change_sound` (0x45C551): the current car's sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`) told of the car's `gear_changed`: at 1 its
//! `drive_slip_speed`-side flag (+0x180) and the car flags (+0x10) are set to 1; at -1 its
//! `velocity`-side flag (+0x198) and +0x10 are set to 1. A reading: a gear change up or down heard.
//!
//! Returns: the sound object
//!
//! Caller: `physics::engine_sound` (its tail). Refuses (before anything) a null sound object when
//! it would be written (the original writes near address 0).

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::session::PlainDword;
use rc2k_core::physics::gearbox::{GearChange, gear_change};

/// On the current car's sound object and the word at this place.
// Converted: `rc2k_core::physics::gearbox::gear_change`.
pub fn gear_change_sound_at(
    o: &crate::sound::store::SoundObject,
    word: &PlainDword,
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

pub fn gear_change_sound(car: &Car) -> Result<(), Refused> {
    gear_change_sound_at(crate::sound::store::current_object(), &car.gear_changed)
}
