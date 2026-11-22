//! When the car numbered `other_car_number` or the current car is listed, the bump band (`sound_bands::bump_band_plain`)
//! gives a band; a band not 0 goes to the current car's sound object as
//! sound 0xE1 with that band, the place and the update flag set (see `physics/car_sounds.rs`). A
//! reading: a sound of another car met.
//!
//! Refuses (before anything) a car number past 7, a null sound object when it is written (the
//! object is looked up first, but only written after the band).

use crate::errors::Refused;
use crate::physics::car_sounds::{cue_plain, listed, object};
use crate::physics::sound_bands::bump_band_plain;
use crate::physics::step_families::SoundInputs;
use rc2k_core::physics::sound_places::{FIRST, SOUND_E1};

// Core: `rc2k_core::physics::sound_places`.
pub fn place_sound_e1_other(
    c: &SoundInputs<'_>,
    other_car_number: u32,
    bump_speed_length: f32,
) -> Result<(), Refused> {
    if !listed(c.frame_work, other_car_number)?
        && !listed(c.frame_work, c.work.borrow().current_car)?
    {
        return Ok(());
    }
    let b = bump_band_plain(bump_speed_length);
    if b == 0 {
        return Ok(());
    }
    cue_plain(c, object(c), FIRST, SOUND_E1, b, None)?;
    Ok(())
}
