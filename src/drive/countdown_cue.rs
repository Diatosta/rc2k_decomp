//! `drive::countdown_cue` (0x45D1A5): `drive::countdown_sound` for the current car when it
//! has a sound slot (`dword_641680[g_currentCar]` not -1), ebx the sound.
//!
//! In:  ebx the sound
//! Out: eax the slot; the rest kept; no flags read
//!
//! Caller: `drive::countdown_beep`. Refuses a current car past 7.

use crate::errors::Refused;
use crate::race::owner_frame::Frame;
use crate::race::work_store::RACE_WORK;

/// The Rust version: eax.
// Converted: `rc2k_core::physics::listed_cars::listed_entry`.
pub fn countdown_cue(frame: &Frame, sound: u32) -> Result<u32, Refused> {
    let (slot, listed) =
        rc2k_core::physics::listed_cars::listed_entry(RACE_WORK.current_car.get(), |car| {
            frame.frame_work.listed_car_sounds[car as usize].get() // dword_641680
        })?;
    if listed {
        crate::drive::countdown_sound::countdown_sound(slot, sound);
    }
    Ok(slot)
}
