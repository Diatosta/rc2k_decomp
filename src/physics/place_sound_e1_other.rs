//! `physics::place_sound_e1_other` (0x45CE32): when the car numbered `dword_6D3CD4` or the current car is
//! listed, `physics::sound_band_3` (ebx); a band not 0 goes into `dword_6D1AD4` and to the
//! current car's sound object with +0x238 0xE1, +0x23C the band, the place at +0x240 ..
//! and +0x10 1 (see `physics/car_sounds.rs`). A reading: a sound of another car met.
//!
//! Caller: `physics::bump_cars` (its tail). Refuses (before anything) a car number past 7, a null
//! sound object when it is written (the object is looked up first, but only written after
//! the band).

use crate::errors::Refused;
use crate::physics::car_sounds::{cue_plain, listed, object};
use crate::physics::sound_band_3::TWIN;
use crate::physics::sound_bands::band_value_plain;
use crate::race::owner::Race;
use rc2k_core::physics::sound_places::{FIRST, SOUND_E1};

/// The Rust version.
// Converted: `rc2k_core::physics::sound_places`.
pub fn place_sound_e1_other(race: &Race) -> Result<(), Refused> {
    if !listed(&race.frame, race.physics.contact.other_car_number.get())?
        && !listed(&race.frame, race.shared.work.current_car.get())?
    {
        return Ok(());
    }
    let b = band_value_plain(race, TWIN);
    if b == 0 {
        return Ok(());
    }
    race.frame.sound_spots.sound_scratch_b.set(b);
    cue_plain(race, object(), FIRST, SOUND_E1, b, None)?;
    Ok(())
}
