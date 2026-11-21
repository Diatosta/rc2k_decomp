//! When the current car is listed, the crash band (`sound_bands::CRASH_BAND`) gives a band that goes to its sound
//! object as sound 0xE5 (second slot) with that band, the place and the update flag set (see
//! `physics/car_sounds.rs`).
//!
//! Refuses (before anything) a car number past 7, a null sound object when it is written.

use crate::errors::Refused;
use crate::physics::car_sounds::{cue_plain, listed, object};
use crate::physics::sound_bands::{CRASH_BAND, band_value_plain};
use crate::physics::step_families::SoundInputs;
use rc2k_core::physics::sound_places::{SECOND, SOUND_E5};

// Core: `rc2k_core::physics::sound_places`.
pub fn place_sound_e5(c: &SoundInputs<'_>) -> Result<(), Refused> {
    if !listed(c.frame_work, c.work.current_car.get())? {
        return Ok(());
    }
    let b = band_value_plain(c, &CRASH_BAND);
    cue_plain(c, object(c), SECOND, SOUND_E5, b, None)?;
    Ok(())
}
