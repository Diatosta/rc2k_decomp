//! Drives the network race: clears the stored cars' outputs, sets the HUD, plays the race, keeps
//! the HUD; then returns this machine's car's race time and whether it did not finish.

use crate::errors::Refused;
use crate::owners::Owners;

/// This machine's car's time (+0x88) and whether it did not finish
/// (+0xB0 is -1: `u32::MAX`, else 0).
pub fn lets_race(o: Owners<'static>) -> Result<(u32, u32), Refused> {
    let race = o.race;
    race.shared.view.get().after_credits.set(0);
    // The eight stored cars' +0xB0 (out).
    for car in &race.car_blocks().cars[..8] {
        car.stop_state.set(0);
    }
    crate::net::set_hud_options::set_hud_options(race, o.net, o.gfx)?;
    { crate::drive::play_race::play_race(race, o) }?;
    crate::net::keep_hud_options::keep_hud_options(&race.shared, o.net)?;
    let block = race
        .shared
        .race_car(race.shared.view.get().local_car.get() as usize)?;
    // The logic is in `rc2k_core::net::record::not_finished`.
    Ok((
        block.stage_time.get(),
        rc2k_core::net::record::not_finished(block.stop_state.get()),
    ))
}
