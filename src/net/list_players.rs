//! `net::list_players` (0x419192): this machine's record sent to the others as its player
//! data (`SetPlayerData`, `byte_52AD64`, 0x10 bytes; its result not looked at); unless the
//! race is on (`dword_52AD84` 1), the players listed afresh (`g_netPlayers` 0, `EnumPlayers`
//! with `net::player_found`); each listed player's record read into its block (+0x24,
//! `GetPlayerData`, the size in `dword_52AD74`, 0x10); the blocks bubble-sorted by their
//! DirectPlay ids (+0x20, unsigned); this machine's player found among them by its id
//! (`g_netPlayerId`) into `g_netPlayer`.
//!
//! In:  nothing
//! Out: true when this machine's player was found (the original: carry clear, the flags of
//!      `test eax, eax` on its id; every register kept); false for the carry: no object,
//!      the listing failed or the player is not among them
//! The search looks at block 0 even with no player listed. More than eight players
//! (possible only with the race on) would sort past the blocks: refused. Caller:
//! `net::list_players_regs`, `net::open_lobby`. DirectPlay's calls from Rust (`net::dplay`).

use crate::errors::{BadInput, Refused};
use crate::net::sort_players::{bubble_blocks, swap_blocks};
use rc2k_core::net::players;

/// The Rust version.
pub fn list_players() -> Result<bool, Refused> {
    if crate::net::owner::net().vars.dplay.get() == 0 {
        return Ok(false);
    }
    crate::net::dp_objects::set_player_data(
        crate::net::owner::net().vars.dplay.get(),
        crate::net::owner::net().vars.player_id.get(),
        &crate::net::owner::net().vars.own.bytes(),
    );
    if crate::net::owner::net().vars.racing.get() != 1 {
        crate::net::owner::net().vars.players.set(0);
        let result = crate::net::dp_objects::enum_players(
            crate::net::owner::net().vars.dplay.get(),
            crate::net::player_found::callback,
            0,
        );
        if result != 0 {
            return Ok(false);
        }
    }
    let players = crate::net::owner::net().vars.players.get();
    if players > players::MOST {
        return Err(BadInput("more than eight players").into());
    }
    for n in 0..players {
        crate::net::owner::net().vars.data_size.set(0x10);
        let player = crate::net::owner::net().vars.player_block(n);
        let mut data = [0u8; 0x10];
        let (result, size) = crate::net::dp_objects::get_player_data(
            crate::net::owner::net().vars.dplay.get(),
            player.id.get(),
            &mut data,
        );
        crate::net::owner::net().vars.data_size.set(size);
        if result == 0 {
            player.record.set_bytes(&data[..size as usize]);
        }
    }
    // Converted: `rc2k_core::net::players` (`bubble` by the ids, `find_own`).
    let id = |n: u32| crate::net::owner::net().vars.player_block(n).id.get();
    bubble_blocks(
        players,
        |k| id(k) > id(k + 1),
        |k| {
            swap_blocks(
                crate::net::owner::net().vars.player_block(k),
                crate::net::owner::net().vars.player_block(k + 1),
            )
        },
    )?;
    match players::find_own(
        crate::net::owner::net().vars.player_id.get(),
        crate::net::owner::net().vars.players.get(),
        |n| crate::net::owner::net().vars.player_block(n).id.get(),
    ) {
        Ok(n) => {
            crate::net::owner::net().vars.player.set(n);
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

/// `net::list_players_regs` (0x417A03): `net::list_players`, then `test eax, eax` on the
/// eax it keeps, inside `pusha` .. `popa`: the carry it gives is lost, so its callers'
/// `jb` after it are never taken (section 16). Callers: `net::lets_rally`,
/// `net::race_wait`, `net::race_over`, `net::lobby_loop`.
pub fn list_players_regs() -> Result<(), Refused> {
    list_players()?;
    Ok(())
}
