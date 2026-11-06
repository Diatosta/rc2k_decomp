//! `net::keep_hud_options` (0x463E39): this machine's car's HUD bits (+0x220 of
//! `g_carBlocks[g_netPlayer]`) kept in `dword_75B00C` for the next race.
//!
//! In:  nothing
//! Out: nothing (the original leaves esi the car's block and eax the bits; its one caller,
//!      `net::lets_race`, sets both next)
//! Caller: `net::lets_race`.

use crate::net::set_hud_options::own_block;

/// The Rust version.
pub fn keep_hud_options() -> Result<(), crate::errors::BadInput> {
    crate::race::owner::race()
        .shared
        .state
        .results
        .hud_bits_current
        .set(crate::race::trap::car_at(own_block())?.option_bits.get());
    Ok(())
}
