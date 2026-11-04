//! `physics::place_sound_e1` (0x45CEF9): when the current car is listed, `physics::sound_band_1` (ebx);
//! a band not 0 goes to its sound object with +0x238 0xE1, +0x23C the band, the place at
//! +0x240 .. and +0x10 1 (see `physics/car_sounds.rs`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_45D8FB`, `sub_45DA14`, `sub_45DC3E`, `sub_45EFF3`. Refuses (before
//! anything) a car number past 7, a null sound object when it is written.

use crate::errors::Refused;
use crate::physics::car_sounds::{listed, object, place, put};
use crate::physics::sound_band_1::TWIN;
use crate::physics::sound_bands::band_value;
use crate::race::work_store::RACE_WORK;

/// The Rust version (the exact one, for `physics::wall_push_back` and
/// `physics::wall_edge_contact`).
pub fn place_sound_e1() -> Result<(), Refused> {
    if !listed(RACE_WORK.current_car.bits())? {
        return Ok(());
    }
    let b = band_value(TWIN);
    if b == 0 {
        return Ok(());
    }
    let o = object();
    put(o, 0x238, 0xE1)?;
    put(o, 0x23C, b)?;
    place(o, 0x240, None)?;
    put(o, 0x10, 1)?;
    Ok(())
}
