//! `physics::place_sound_e1_other` (0x45CE32): when the car numbered `dword_6D3CD4` or the current car is
//! listed, `physics::sound_band_3` (ebx); a band not 0 goes into `dword_6D1AD4` and to the
//! current car's sound object with +0x238 0xE1, +0x23C the band, the place at +0x240 ..
//! and +0x10 1 (see `physics/car_sounds.rs`). A reading: a sound of another car met.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_45F7B1` (its tail). Refuses (before anything) a car number past 7, a null
//! sound object when it is written (the object is looked up first, but only written after
//! the band).

use crate::errors::Refused;
use crate::physics::car_sounds::{cue_plain, listed, object};
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::sound_band_3::TWIN;
use crate::physics::sound_bands::band_value_plain;
use crate::race::sound_spots_store::RACE_SOUND_SPOTS;
use crate::race::work_store::RACE_WORK;
use rc2k_core::physics::sound_places::{FIRST, SOUND_E1};

/// The Rust version.
// Converted: `rc2k_core::physics::sound_places`.
pub unsafe fn place_sound_e1_other() -> Result<(), Refused> {
    if !listed(CONTACT_WORK.other_car_number.get())? && !listed(RACE_WORK.current_car.get())? {
        return Ok(());
    }
    let scratch = RACE_SOUND_SPOTS.band_scratch.get();
    let b = unsafe { band_value_plain(TWIN) };
    if b == 0 {
        return Ok(());
    }
    // The band left `dword_6D1AEC` written: put back if the object refuses.
    let o = match object() {
        Ok(o) => o,
        Err(e) => {
            RACE_SOUND_SPOTS.band_scratch.set_bits(scratch);
            return Err(e);
        }
    };
    unsafe {
        RACE_SOUND_SPOTS.sound_scratch_b.set(b);
        cue_plain(o, FIRST, SOUND_E1, b, None);
    }
    Ok(())
}
