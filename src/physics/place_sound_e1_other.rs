//! `physics::place_sound_e1_other` (0x45CE32): when the car numbered `other_car_number` or the
//! current car is listed, `physics::sound_band_3` gives a band; a band not 0 goes into
//! `sound_scratch_b` and to the current car's sound object as sound 0xE1 with that band, the
//! place and the update flag set (see `physics/car_sounds.rs`). A reading: a sound of another car
//! met.
//!
//! Caller: `physics::bump_cars` (its tail). Refuses (before anything) a car number past 7, a null
//! sound object when it is written (the object is looked up first, but only written after
//! the band).

use crate::errors::Refused;
use crate::physics::car_sounds::{cue_plain, listed, object};
use crate::physics::sound_band_3::TWIN;
use crate::physics::sound_bands::band_value_plain;
use crate::physics::step_families::SoundInputs;
use rc2k_core::physics::sound_places::{FIRST, SOUND_E1};

// Converted: `rc2k_core::physics::sound_places`.
pub fn place_sound_e1_other(c: &SoundInputs<'_>) -> Result<(), Refused> {
    if !listed(c.frame_work, c.contact.other_car_number.get())?
        && !listed(c.frame_work, c.work.current_car.get())?
    {
        return Ok(());
    }
    let b = band_value_plain(c, TWIN);
    if b == 0 {
        return Ok(());
    }
    c.spots.sound_scratch_b.set(b);
    cue_plain(c, object(), FIRST, SOUND_E1, b, None)?;
    Ok(())
}
