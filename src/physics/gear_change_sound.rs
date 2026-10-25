//! `physics::gear_change_sound` (0x45C551): the current car's sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`, in edi) told of `g_car` +0x238: at 1 its
//! +0x180 and +0x10 set to 1; at -1 its +0x198 and +0x10 set to 1. A reading: a gear
//! change up or down heard.
//!
//! In:  nothing
//! Out: edi the sound object; the rest kept; no flags read (the caller returns)
//!
//! Caller: `sub_43DC72` (its tail). Refuses (before anything) a null sound object when it
//! would be written (the original writes near address 0).

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::CAR;
use crate::race::work_store::RACE_WORK;
use rc2k_core::physics::gearbox::{GearChange, gear_change};

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version on the car number, the object table and the word at these places.
// Converted: `rc2k_core::physics::gearbox::gear_change`.
pub unsafe fn gear_change_sound_at(current: u32, table: u32, word: u32) -> Result<(), Refused> {
    unsafe {
        let object = at(table + 4 * (at(current) & 7));
        if let Some(change) = gear_change(at(word)) {
            if object == 0 {
                return Err(BadInput("a null sound object").into());
            }
            let o = crate::sound::store::SoundObject::at(object);
            match change {
                GearChange::Up => o.slots[2].start.set(1),
                GearChange::Down => o.slots[3].start.set(1),
            }
            o.new_revs.set(1);
        }
        Ok(())
    }
}

/// The Rust version.
pub unsafe fn gear_change_sound() -> Result<(), Refused> {
    unsafe {
        gear_change_sound_at(
            RACE_WORK.current_car.addr() as u32,
            crate::sound::store::RACE_SOUNDS.table.as_ptr() as usize as u32,
            CAR.gear_changed.addr() as u32,
        )
    }
}
