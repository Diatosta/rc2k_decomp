//! A remote car's step in a network race: while it has glide steps left, one less and a 32nd
//! of the way to the position received is added to its position. Calls nothing.

use crate::net::apply_message::{glide_steps, place, three, wanted};
use crate::net::owner::net_race;
use crate::race::owner::Race;
use rc2k_core::net::race;
pub fn follow_remote(race: &Race) -> Result<(), crate::errors::BadInput> {
    let car = race.shared.car();
    // The logic is in `rc2k_core::net::race::glide` (the block read only with the race on).
    if !net_race().is_on() {
        return Ok(());
    }
    let steps = glide_steps(car).get();
    if steps == 0 {
        return Ok(());
    }
    let step = three(wanted(car));
    if let Some((left, new_place)) = race::glide(true, steps, three(place(car)), step) {
        glide_steps(car).set(left);
        for (cell, v) in place(car).iter().zip(new_place) {
            cell.set(v);
        }
    }
    Ok(())
}
