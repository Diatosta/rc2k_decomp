//! `net::race_over` (0x41732E): this player's race done: `dword_52AD78` and `dword_52AD7C` 0,
//! the race's time (eax) to the record (+8) and added to its total (+0xC), its state
//! "raced" (bits 0 and 1 off, bit 2 on), "after a race" (`dword_52AD80` 1), and the players
//! listed (`net::list_players_regs`: the record sent).
//!
//! In:  `time` (eax: the race's time)
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! Caller: `net::lets_rally`.

use crate::errors::Refused;
use crate::net::lobby::{
    ALL_RACED, BACK_FROM_RACE, OWN, OWN_TIME, OWN_TOTAL, RACE_OVER, g, gb, p, pb,
};

/// The Rust version.
pub fn race_over(time: u32) -> Result<(), Refused> {
    p(ALL_RACED, 0);
    p(RACE_OVER, 0);
    p(OWN_TIME, time);
    p(OWN_TOTAL, g(OWN_TOTAL).wrapping_add(time));
    // Converted: `rc2k_core::net::players::raced`.
    pb(OWN, rc2k_core::net::players::raced(gb(OWN)));
    p(BACK_FROM_RACE, 1);
    crate::net::list_players::list_players_regs()
}
