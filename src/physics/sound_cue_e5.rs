//! `physics::sound_cue_e5` (0x45D028): when the current car's entry in `dword_641680` is
//! not -1, its sound object (`g_raceSoundObjects[g_currentCar & 7]`) gets +0x24C 0xE5,
//! +0x250 0 and +0x10 1 (a reading: a sound to play). All in `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller returns)
//!
//! Caller: `sub_45EDB5` (its tail). Refuses (before anything) a current car past 7, or a
//! null sound object when written.

use crate::errors::Refused;
use crate::race::owner_frame::Frame;
use crate::race::work_store::RACE_WORK;
use crate::session::Dword;
use crate::sound::store::SoundObject;
use rc2k_core::physics::listed_cars::{CUE_E5, cue_e5_object};

/// The core: the car number and the listed table; the object is the car's number
/// (`g_raceSoundObjects[car & 7]`).
// Converted: `rc2k_core::physics::listed_cars::{cue_e5_object, CUE_E5}`.
pub fn sound_cue_e5_at(current: &Dword, listed: &[Dword; 8]) -> Result<(), Refused> {
    // `cue_e5_object` refuses a car past 7 before it asks for its entry.
    let object = cue_e5_object(current.get(), |car| listed[car as usize].get())?;
    if let Some(number) = object {
        let o = SoundObject::number(number)?;
        let [(_, sound), (_, volume), (_, play)] = CUE_E5;
        o.cue_sound.set(sound);
        o.cue_volume.set(volume);
        // +0x10: the play switch, the cell the revs' new reading flag shares.
        o.new_revs.set(play);
    }
    Ok(())
}

/// The Rust version.
pub fn sound_cue_e5(frame: &Frame) -> Result<(), Refused> {
    sound_cue_e5_at(&RACE_WORK.current_car, &frame.frame_work.listed_car_sounds)
}
