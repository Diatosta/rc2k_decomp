//! `drive::countdown_sound` for the current car when it has a sound slot. Refuses a current car
//! past 7.

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn countdown_cue(
    race: &Race,
    sound: u32,
    audio: &crate::audio::owner::Audio,
) -> Result<u32, Refused> {
    let (slot, listed) = rc2k_core::physics::listed_cars::listed_entry(
        race.shared.work.get().current_car.get(),
        |car| {
            race.frame.frame_work.get().listed_car_sounds[car as usize].get() // FrameWork.listed_car_sounds[0]
        },
    )?;
    if listed {
        crate::drive::countdown_sound::countdown_sound(slot, sound, audio);
    }
    Ok(slot)
}
