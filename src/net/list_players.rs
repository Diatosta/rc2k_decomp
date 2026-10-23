//! `net::list_players` (0x419192): this machine's record sent to the others as its player
//! data (`SetPlayerData`, `byte_52AD64`, 0x10 bytes; its result not looked at); unless the
//! race is on (`dword_52AD84` 1), the players listed afresh (`g_netPlayers` 0, `EnumPlayers`
//! with `net::player_found`); each listed player's record read into its block (+0x24,
//! `GetPlayerData`, the size in `dword_52AD74`, 0x10); the blocks bubble-sorted by their
//! DirectPlay ids (+0x20, unsigned); this machine's player found among them by its id
//! (`g_netPlayerId`) into `g_netPlayer`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if there is no object, the
//!      listing failed or this machine's player is not among them, else the flags of
//!      `test eax, eax` on its id
//! The search looks at block 0 even with no player listed. More than eight players
//! (possible only with the race on) would sort past the blocks: refused. Caller:
//! `net::list_players_regs`, `net::open_lobby`. DirectPlay's calls from Rust (`net::dplay`),
//! the callback the game's address (its hook). Shadow mode runs the original alone.
//! Self-test: a copy whose DirectPlay is a test object (its `EnumPlayers` listing players),
//! against the Rust version: no object, failing, racing, found, not found.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::field;
use crate::framework::Regs;
use crate::net::dplay::{ENUM_PLAYERS, GET_PLAYER_DATA, SET_PLAYER_DATA, call};
use crate::net::lobby::{
    DPLAY, OWN, PLAYER, PLAYER_ID, PLAYERS, Part, RACING, a, block, cmp_flags, g, lobby_entry, p,
    test_eax,
};
use crate::net::sort_players::swap_blocks;
use crate::net::types::NetPlayerBlock;
use rc2k_core::net::players;

const DATA_SIZE: usize = 0x52AD74; // dword_52AD74

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn list_players(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs| Regs {
        eflags: r.eflags,
        ..*entry
    };
    cmp_flags(&mut r, g(DPLAY), 0);
    if g(DPLAY) == 0 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    unsafe {
        r.eax = call(DPLAY, SET_PLAYER_DATA, &[g(PLAYER_ID), a(OWN), 0x10, 0]);
        if g(RACING) != 1 {
            p(PLAYERS, 0);
            r.eax = call(
                DPLAY,
                ENUM_PLAYERS,
                &[
                    0,
                    crate::net::player_found::callback as *const () as usize as u32,
                    0,
                    0,
                ],
            );
            test_eax(&mut r);
            if r.eax != 0 {
                r.set_carry(true);
                return Ok(left(&r));
            }
        }
        let players = g(PLAYERS);
        if players > players::MOST {
            return Err(BadInput("more than eight players").into());
        }
        for n in 0..players {
            p(DATA_SIZE, 0x10);
            let at = block(n);
            r.eax = call(
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
    r.eax = g(PLAYER_ID);
    match players::find_own(r.eax, g(PLAYERS), |n| {
        get(block(n) + field!(NetPlayerBlock, id))
    }) {
        Ok(n) => {
            p(PLAYER, n);
            test_eax(&mut r);
        }
        Err(n) => {
            cmp_flags(&mut r, n, g(PLAYERS));
            r.set_carry(true);
        }
    }
    Ok(left(&r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, list_players)
}
