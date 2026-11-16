//! Keeps this machine's car's HUD bits as the player's saved HUD bits for the next race.

use crate::net::set_hud_options::own_block;
use crate::race::owner_shared::RaceShared;
pub fn keep_hud_options(
    shared: &RaceShared,
    net: &crate::net::owner::Net,
) -> Result<(), crate::errors::BadInput> {
    shared
        .state
        .results
        .hud_bits_current
        .set(own_block(shared, net.race_slot.player())?.option_bits.get());
    Ok(())
}
