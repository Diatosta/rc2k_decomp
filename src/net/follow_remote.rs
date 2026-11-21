//! A remote car's step in a network race: while it has glide steps left, one less and a 32nd
//! of the way to the position received is added to its position. Calls nothing.

use crate::net::apply_message::{glide_steps, place, wanted};
use crate::net::owner::Net;
use crate::race::owner::Race;
use rc2k_core::net::race;
pub fn follow_remote(race: &Race, net: &Net) -> Result<(), crate::errors::BadInput> {
    let car = race.shared.car();
    // The logic is in `rc2k_core::net::race::glide` (the block read only with the race on).
    if !net.race_slot.is_on() {
        return Ok(());
    }
    let steps = glide_steps(car).get();
    if steps == 0 {
        return Ok(());
    }
    let step = wanted(car).bits();
    if let Some((left, new_place)) = race::glide(true, steps, place(car).bits(), step) {
        glide_steps(car).set(left);
        place(car).set_bits(new_place);
    }
    Ok(())
}
