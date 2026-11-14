//! DirectPlay's `EnumPlayers` callback: while fewer than eight players are listed, the player's
//! block gets its id and short name. Returns 1 to go on, 0 to stop (eight listed).

use crate::errors::Refused;
use crate::net::owner::net_race;
use crate::net::store::NetShared;

/// The player's id and short name.
pub fn player_found(vars: &NetShared, id: u32, short: &[u8]) -> Result<u32, Refused> {
    let Some(count) = rc2k_core::net::lists::player_found(net_race().players()) else {
        return Ok(0);
    };
    net_race().set_players(count + 1);
    let player = vars.player_block(count)?;
    player.id.set(id);
    let written = crate::strings::copy_printable::printable_from(short, 0x1F)?;
    player.name.write_at(0, &written, "a player's name")?;
    Ok(1)
}

pub fn callback(vars: &NetShared, id: u32, short: &[u8]) -> u32 {
    crate::net::lobby::callback_entry("net::player_found", 0x41929E, || {
        player_found(vars, id, short)
    })
}
