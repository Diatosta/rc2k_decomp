//! `physics::place_sound_e1` (0x45CEF9): when the current car is listed, `physics::sound_band_1` (ebx);
//! a band not 0 goes to its sound object with +0x238 0xE1, +0x23C the band, the place at
//! +0x240 .. and +0x10 1 (see `physics/car_sounds.rs`).
//!
//! Caller: `physics::wall_push_back`, `physics::wall_edge_contact`,
//! `physics::crossed_edge_contact`, `physics::ground_contact`. Refuses (before anything) a car
//! number past 7, a null sound object when it is written.

use crate::errors::Refused;
use crate::physics::car_sounds::{listed, object, place};
use crate::physics::sound_band_1::TWIN;
use crate::physics::sound_bands::band_value;
use crate::race::owner::Race;

/// The exact version, for `physics::wall_push_back` and `physics::wall_edge_contact`.
pub fn place_sound_e1(race: &Race) -> Result<(), Refused> {
    if !listed(&race.frame, race.shared.work.current_car.bits())? {
        return Ok(());
    }
    let b = band_value(race, TWIN);
    if b == 0 {
        return Ok(());
    }
    let o = object();
    o.hit_sound.set(0xE1);
    o.hit_volume.set(b);
    place(race, &o.hit_place, None);
    // +0x10: the play switch, the cell the revs' new reading flag shares.
    o.new_revs.set(1);
    Ok(())
}
