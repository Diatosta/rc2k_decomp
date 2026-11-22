//! When the current car is listed, the contact band (`sound_bands::CONTACT_BAND`) gives a band; a band not 0 goes to the
//! car's sound object as sound 0xE1 with that band, the car's place and the update flag set (see
//! `physics/car_sounds.rs`).
//!
//! Refuses (before anything) a car number past 7, a null sound object when it is written.

use crate::errors::Refused;
use crate::physics::car_sounds::{listed, object, place};
use crate::physics::sound_bands::{CONTACT_BAND, band_value};
use crate::physics::step_families::SoundInputs;

/// The exact version, for `physics::wall_push_back` and `physics::wall_edge_contact`.
pub fn place_sound_e1(c: &SoundInputs<'_>) -> Result<(), Refused> {
    if !listed(c.frame_work, c.work.borrow().current_car)? {
        return Ok(());
    }
    let b = band_value(c, &CONTACT_BAND);
    if b == 0 {
        return Ok(());
    }
    let o = object(c);
    o.borrow_mut().hit_sound = 0xE1;
    o.borrow_mut().hit_volume = b;
    let at = place(
        c.numbers,
        &c.view_turn,
        c.heard_turn,
        c.car,
        c.listener,
        None,
    );
    o.borrow_mut().hit_place = at;
    // The play switch, the cell the revs' new reading flag shares.
    o.borrow_mut().new_revs = 1;
    Ok(())
}
