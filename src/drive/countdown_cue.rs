//! `drive::countdown_sound` for the current car when it has a sound slot. Refuses a current car
//! past 7.

use crate::errors::Refused;
use crate::race::owner::Race;

pub(super) fn countdown_cue(
    race: &Race,
    sound: u32,
    audio: &crate::audio::owner::Audio,
) -> Result<u32, Refused> {
    let (slot, listed) =
        rc2k_core::physics::listed_cars::listed_entry(race.shared.work().current_car, |car| {
            race.frame.frame_work.borrow().listed_car_sounds[car as usize]
        })?;
    if listed {
        crate::drive::countdown_sound::countdown_sound(slot, sound, audio);
    }
    Ok(slot)
}
