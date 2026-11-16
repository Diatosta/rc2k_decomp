//! When the current car's entry in `listed_car_sounds` is not -1, its sound object (in the race's
//! sound objects) gets the cue 0xE5, a cue count of 0 and its update flag set (a reading: a sound
//! to play).
//!
//! Caller: `physics::crash_contact`. Refuses (before anything) a current car past 7, or a null
//! sound object when written.

use crate::errors::Refused;
use crate::memory::session::PlainDword;
use crate::race::frame_store::FrameWork;
use crate::race::work_store::RaceWork;
use crate::sound::store::SoundObject;
use rc2k_core::physics::listed_cars::{CUE_E5, cue_e5_object};

/// The core: the car number and the listed table; the object is the car's sound object (by car
/// number & 7).
// Core: `rc2k_core::physics::listed_cars::{cue_e5_object, CUE_E5}`.
pub fn sound_cue_e5_at(
    current: &impl crate::memory::session::DwordLike,
    listed: &[PlainDword; 8],
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    // `cue_e5_object` refuses a car past 7 before it asks for its entry.
    let object = cue_e5_object(current.get(), |car| listed[car as usize].get())?;
    if let Some(number) = object {
        let o = SoundObject::of(sound, number)?;
        let [(_, sound), (_, volume), (_, play)] = CUE_E5;
        o.cue_sound.set(sound);
        o.cue_volume.set(volume);
        // The play switch, the cell the revs' new reading flag shares.
        o.new_revs.set(play);
    }
    Ok(())
}

pub fn sound_cue_e5(
    frame_work: &FrameWork,
    work: &RaceWork,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    sound_cue_e5_at(&work.current_car, &frame_work.listed_car_sounds, sound)
}
