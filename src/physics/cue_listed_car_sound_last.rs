//! `physics::cue_listed_car_sound_last` (0x45D1D3): `physics::cue_listed_car_sound`'s twin for
//! the pace notes: takes the current car's entry in `listed_car_sounds`; unless it is -1,
//! `physics::cue_car_sound_last` runs with it.
//!
//! Takes: the cue
//! Returns: the entry
//!
//! Caller: `physics::queue_pace_note`. Refuses (before anything) a current car past 7.

use crate::errors::Refused;
use crate::race::frame_store::FrameWork;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::listed_cars::listed_entry;

/// `cue` for the listed car, if it is listed: its entry returned.
// Converted: `rc2k_core::physics::listed_cars::listed_entry`.
pub fn cue_listed_car_sound_last(
    frame_work: &FrameWork,
    work: &RaceWork,
    cue: u32,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<u32, Refused> {
    let (entry, listed) = listed_entry(work.current_car.get(), |car| {
        frame_work.listed_car_sounds[car as usize].get()
    })?;
    if listed {
        crate::physics::cue_car_sound_last::cue_car_sound_last(entry, cue, sound, audio)?;
    }
    Ok(entry)
}
