//! `net::race_over` (0x41732E): this player's race done: `dword_52AD78` and `dword_52AD7C` 0,
//! the race's time (eax) to the record (+8) and added to its total (+0xC), its state
//! "raced" (bits 0 and 1 off, bit 2 on), "after a race" (`dword_52AD80` 1), and the players
//! listed (`net::list_players_regs`: the record sent).
//!
//! In:  `time` (eax: the race's time)
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! Caller: `net::lets_rally`.

use crate::errors::Refused;
use crate::net::store::{NET, PlayerRecord};

/// The Rust version.
pub fn race_over(time: u32) -> Result<(), Refused> {
    NET.all_raced.set(0);
    NET.race_over.set(0);
    NET.own.race_time.set(time);
    NET.own.total.set(NET.own.total.get().wrapping_add(time));
    // Converted: `rc2k_core::net::players::raced`.
    NET.own.head.set(
        PlayerRecord::STATE,
        rc2k_core::net::players::raced(NET.own.state()),
    );
    NET.after_race.set(1);
    crate::net::list_players::list_players_regs()
}
