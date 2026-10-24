//! `net::race_wait` (0x4171FE): one turn of waiting for the race to start: this player's
//! state made "waiting" (bit 1 off, bit 2 on), its time -1; once a second the players listed
//! (`net::list_players_regs`); on the host, with every player waiting (state bit 2, those
//! with 0x80 left out), the sync of `net::lobby_loop` with 'P' messages, and when all have
//! answered `dword_52AD7C` 1.
//!
//! In:  nothing
//! Out: `dword_52AD7C` (eax; 1: go). The original keeps the rest (`pusha` .. `popa`) and
//!      sets the carry with no players on the host (or a failed listing, which never comes);
//!      its one caller, `net::lets_rally`, reads eax only (`lq -s 0x4171FE`).
//! Caller: `net::lets_rally`, once a frame. The sync's registers are its own scratch.

use crate::errors::Refused;
use crate::net::lobby::{CLOCK_A, OWN, OWN_TIME, PLAYERS, RACE_OVER, g, gb, p, pb};
use crate::net::lobby_loop::{all_states, sync};
use rc2k_core::net::players;

/// The Rust version.
pub fn race_wait() -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::players` (`waiting`, `all_in_state` in `all_states`).
    pb(OWN, players::waiting(gb(OWN)));
    p(OWN_TIME, u32::MAX);
    let now = crate::system::tick_count::tick_count();
    if (now.wrapping_sub(g(CLOCK_A)).wrapping_sub(1000) as i32) >= 0 {
        p(CLOCK_A, now);
        crate::net::list_players::list_players_regs()?;
    }
    if gb(OWN) & players::HOST != 0 {
        if g(PLAYERS) == 0 {
            return Ok(g(RACE_OVER));
        }
        if all_states(players::WAITING, g(PLAYERS)) == players::WAITING && sync(b'P')? {
            p(RACE_OVER, 1);
        }
    }
    Ok(g(RACE_OVER))
}
