//! `net::lets_race` (0x4640AA): the network race driven: a race variable and the eight stored
//! cars' +0x74 (`g_carBlockStore.out`) 0, the HUD set (`net::set_hud_options`), the race
//! (`drive::play_race`), the HUD kept (`net::keep_hud_options`); then this machine's car
//! (`g_carBlocks[local car]`): its +0x88 (the race's time, it seems) and 0, or -1
//! when its +0xB0 is -1.
//!
//! Returns the time and whether the car did not finish.
//! Caller: `net::lets_rally`.

use crate::errors::Refused;
use crate::net::store::NetShared;
use crate::owners::Owners;

/// This machine's car's time (+0x88) and whether it did not finish
/// (+0xB0 is -1: `u32::MAX`, else 0).
pub fn lets_race(o: Owners<'static>, vars: &NetShared) -> Result<(u32, u32), Refused> {
    let race = o.race;
    race.shared.view.get().after_credits.set(0);
    // The eight stored cars' +0xB0 (out).
    for car in &race.car_blocks().cars[..8] {
        car.stop_state.set(0);
    }
    crate::net::set_hud_options::set_hud_options(race, o.gfx, vars)?;
    { crate::drive::play_race::play_race(race, o) }?;
    crate::net::keep_hud_options::keep_hud_options(&race.shared, vars)?;
    let block = race
        .shared
        .race_car(race.shared.view.get().local_car.get() as usize)?;
    // Converted: `rc2k_core::net::record::not_finished`.
    Ok((
        block.stage_time.get(),
        rc2k_core::net::record::not_finished(block.stop_state.get()),
    ))
}
