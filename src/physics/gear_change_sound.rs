//! `physics::gear_change_sound` (0x45C551): the current car's sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`, in edi) told of `g_car` +0x238: at 1 its
//! +0x180 and +0x10 set to 1; at -1 its +0x198 and +0x10 set to 1. A reading: a gear
//! change up or down heard.
//!
//! Returns: the sound object
//!
//! Caller: `physics::engine_sound` (its tail). Refuses (before anything) a null sound object when
//! it would be written (the original writes near address 0).

use crate::errors::Refused;
use crate::race::owner_shared::Shared;
use crate::session::Dword;
use rc2k_core::physics::gearbox::{GearChange, gear_change};

/// The Rust version on the current car's sound object and the word at this place.
// Converted: `rc2k_core::physics::gearbox::gear_change`.
pub fn gear_change_sound_at(
    o: &crate::sound::store::SoundObject,
    word: &Dword,
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

/// The Rust version.
pub fn gear_change_sound(shared: &Shared) -> Result<(), Refused> {
    gear_change_sound_at(
        crate::sound::store::current_object(),
        &shared.car.gear_changed,
    )
}
