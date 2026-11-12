//! `net::player_found` (0x41929E): DirectPlay's `EnumPlayers` callback: while fewer than eight
//! are listed (`g_netPlayers`), the player's block of `g_netReceived` gets its id (+0x20) and
//! its short name (+0, `strings::copy_printable`, 0x1F room), and the enumeration goes on;
//! with eight it stops.
//!
//! Takes (stdcall) the player's id, its type, its name (a DPNAME: +8 the short name), the flags
//! and the context. Returns 1 to go on, 0 to stop.
//! Caller: DirectPlay, from `net::list_players` (`callback`, a `stdcall` Rust function).

use crate::errors::Refused;
use crate::net::store::NetVars;

/// The player's id and short name.
pub fn player_found(vars: &NetVars, id: u32, short: &[u8]) -> Result<u32, Refused> {
    let Some(count) = rc2k_core::net::lists::player_found(vars.players.get()) else {
        return Ok(0);
    };
    vars.players.set(count + 1);
    let player = vars.player_block(count)?;
    player.id.set(id);
    let written = crate::strings::copy_printable::printable_from(short, 0x1F)?;
    player.name.write_at(0, &written, "a player's name")?;
    Ok(1)
}

pub fn callback(vars: &NetVars, id: u32, short: &[u8]) -> u32 {
    crate::net::lobby::callback_entry("net::player_found", 0x41929E, || {
        player_found(vars, id, short)
    })
}
