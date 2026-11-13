//! `net::list_players` (0x419192): this machine's record sent to the others as its player
//! data (`SetPlayerData`, `own`, 0x10 bytes; its result not looked at); unless the
//! race is on (`racing` 1), the players listed afresh (`g_netPlayers` 0, `EnumPlayers`
//! with `net::player_found`); each listed player's record read into its block (+0x24,
//! `GetPlayerData`, the size in `data_size`, 0x10); the blocks bubble-sorted by their
//! DirectPlay ids (+0x20, unsigned); this machine's player found among them by its id
//! (`g_netPlayerId`) into `g_netPlayer`.
//!
//! Returns true when this machine's player was found; false when there is no object, the
//! listing failed or the player is not among them.
//! The search looks at block 0 even with no player listed. More than eight players
//! (possible only with the race on) would sort past the blocks: refused. Caller:
//! `net::list_players_regs`, `net::open_lobby`. DirectPlay's calls from Rust (`net::dplay`).

use crate::errors::{BadInput, Refused};
use crate::net::owner::Net;
use crate::net::sort_players::{bubble_blocks, swap_blocks};
use rc2k_core::net::players;
pub fn list_players(net: &Net) -> Result<bool, Refused> {
    let vars = &net.vars;
    if vars.dplay.get() == 0 {
        return Ok(false);
    }
    crate::net::dp_objects::set_player_data(
        vars.dplay.get(),
        vars.player_id.get(),
        &vars.own.bytes(),
    );
    if net.lobby().racing.get() != 1 {
        vars.players.set(0);
        let result = crate::net::dp_objects::enum_players(
            vars.dplay.get(),
            &|id, _kind, short, _flags, _context| {
                crate::net::player_found::callback(vars, id, short)
            },
            0,
        );
        if result != 0 {
            return Ok(false);
        }
    }
    let players = vars.players.get();
    if players > players::MOST {
        return Err(BadInput("more than eight players").into());
    }
    for n in 0..players {
        net.lobby().data_size.set(0x10);
        let player = vars.player_block(n)?;
        let mut data = [0u8; 0x10];
        let (result, size) =
            crate::net::dp_objects::get_player_data(vars.dplay.get(), player.id.get(), &mut data);
        net.lobby().data_size.set(size);
        if result == 0 {
            player.record.set_bytes(&data[..size as usize]);
        }
    }
    // Converted: `rc2k_core::net::players` (`bubble` by the ids, `find_own`).
    let id = |n: u32| vars.player_block_or_end(n).id.get();
    bubble_blocks(
        players,
        |k| id(k) > id(k + 1),
        |k| swap_blocks(vars.player_block(k)?, vars.player_block(k + 1)?),
    )?;
    match players::find_own(vars.player_id.get(), vars.players.get(), |n| {
        vars.player_block_or_end(n).id.get()
    }) {
        Ok(n) => {
            vars.player.set(n);
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

/// `net::list_players_regs`: `net::list_players`, then a test of the result kept inside a
/// register save: the carry it gives is lost, so its callers'
/// `jb` after it are never taken (section 16). Callers: `net::lets_rally`,
/// `net::race_wait`, `net::race_over`, `net::lobby_loop`.
pub fn list_players_regs(net: &Net) -> Result<(), Refused> {
    list_players(net)?;
    Ok(())
}
