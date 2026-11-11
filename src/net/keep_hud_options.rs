//! `net::keep_hud_options` (0x463E39): this machine's car's HUD bits (+0x220 of
//! `g_carBlocks[g_netPlayer]`) kept in the player's saved HUD bits for the next race.
//!
//! Caller: `net::lets_race`.

use crate::net::set_hud_options::own_block;
use crate::net::store::NetVars;
use crate::race::owner_shared::RaceShared;
pub fn keep_hud_options(
    shared: &RaceShared,
    vars: &NetVars,
) -> Result<(), crate::errors::BadInput> {
    shared.state.results.hud_bits_current.set(
        crate::race::trap::car_at(own_block(shared, vars))?
            .option_bits
            .get(),
    );
    Ok(())
}
