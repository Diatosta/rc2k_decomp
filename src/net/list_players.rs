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
use crate::field;
use crate::net::dplay::{ENUM_PLAYERS, GET_PLAYER_DATA, SET_PLAYER_DATA, call};
use crate::net::lobby::{DPLAY, OWN, PLAYER, PLAYER_ID, PLAYERS, RACING, a, block, g, p};
use crate::net::sort_players::swap_blocks;
use crate::net::types::NetPlayerBlock;
use rc2k_core::net::players;

const DATA_SIZE: usize = 0x52AD74; // dword_52AD74

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The Rust version.
pub fn list_players() -> Result<bool, Refused> {
    if g(DPLAY) == 0 {
        return Ok(false);
    }
    unsafe {
        call(DPLAY, SET_PLAYER_DATA, &[g(PLAYER_ID), a(OWN), 0x10, 0]);
        if g(RACING) != 1 {
            p(PLAYERS, 0);
            let result = call(
                DPLAY,
                ENUM_PLAYERS,
                &[
                    0,
                    crate::net::player_found::callback as *const () as usize as u32,
                    0,
                    0,
                ],
            );
            if result != 0 {
                return Ok(false);
            }
        }
        let players = g(PLAYERS);
        if players > players::MOST {
            return Err(BadInput("more than eight players").into());
        }
        for n in 0..players {
            p(DATA_SIZE, 0x10);
            let at = block(n);
            call(
                DPLAY,
                GET_PLAYER_DATA,
                &[
                    get(at + field!(NetPlayerBlock, id)),
                    at + field!(NetPlayerBlock, record),
                    a(DATA_SIZE),
                    0,
                ],
            );
        }
        // Converted: `rc2k_core::net::players` (`bubble` by the ids, `find_own`).
        let id = |n: u32| get(block(n) + field!(NetPlayerBlock, id));
        players::bubble(players, |k| id(k) > id(k + 1), |k| swap_blocks(block(k)));
    }
    match players::find_own(g(PLAYER_ID), g(PLAYERS), |n| {
        get(block(n) + field!(NetPlayerBlock, id))
    }) {
        Ok(n) => {
            p(PLAYER, n);
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
