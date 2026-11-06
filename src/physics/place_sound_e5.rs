//! `physics::place_sound_e5` (0x45CF95): when the current car is listed, `physics::sound_band_4` (ebx)
//! goes to its sound object with +0x24C 0xE5, +0x250 the band, the place at +0x254 .. and
//! +0x10 1 (see `physics/car_sounds.rs`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_45EDB5`. Refuses (before anything) a car number past 7, a null sound object
//! when it is written.

use crate::errors::Refused;
use crate::physics::car_sounds::{cue_plain, listed, object};
use crate::physics::sound_band_4::TWIN;
use crate::physics::sound_bands::band_value_plain;
use crate::race::owner::Race;
use rc2k_core::physics::sound_places::{SECOND, SOUND_E5};

/// The Rust version.
// Converted: `rc2k_core::physics::sound_places`.
pub fn place_sound_e5(race: &Race) -> Result<(), Refused> {
    if !listed(&race.frame, race.shared.work.current_car.bits())? {
        return Ok(());
    }
    let b = band_value_plain(&race.frame, TWIN);
    cue_plain(race, object(), SECOND, SOUND_E5, b, None)?;
    Ok(())
}
