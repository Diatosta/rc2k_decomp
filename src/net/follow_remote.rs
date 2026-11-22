//! A remote car's step in a network race: while it has glide steps left, one less and a 32nd
//! of the way to the position received is added to its position. Calls nothing.

use crate::net::owner::Net;
use crate::race::owner::Race;
use rc2k_core::net::race;
pub fn follow_remote(race: &Race, net: &Net) -> Result<(), crate::errors::BadInput> {
    let mut car = race.shared.car_mut();
    // The logic is in `rc2k_core::net::race::glide` (the block read only with the race on).
    if !net.race_slot.is_on() {
        return Ok(());
    }
    let steps = car.glide_steps_left;
    if steps == 0 {
        return Ok(());
    }
    let step = car.glide_target.map(f32::to_bits);
    if let Some((left, new_place)) = race::glide(true, steps, car.position.map(f32::to_bits), step)
    {
        car.glide_steps_left = left;
        car.position = new_place.map(f32::from_bits);
    }
    Ok(())
}
