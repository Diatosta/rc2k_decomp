//! `net::race_over` (0x41732E): this player's race done: `dword_52AD78` and `dword_52AD7C` 0,
//! the race's time (eax) to the record (+8) and added to its total (+0xC), its state
//! "raced" (bits 0 and 1 off, bit 2 on), "after a race" (`dword_52AD80` 1), and the players
//! listed (`net::list_players_regs`: the record sent).
//!
//! In:  `time` (eax: the race's time)
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! Caller: `net::lets_rally`.

use crate::errors::Refused;
use crate::net::store::PlayerRecord;

/// The Rust version.
pub fn race_over(time: u32) -> Result<(), Refused> {
    crate::net::owner::net().vars.all_raced.set(0);
    crate::net::owner::net().vars.race_over.set(0);
    crate::net::owner::net().vars.own.race_time.set(time);
    crate::net::owner::net().vars.own.total.set(
        crate::net::owner::net()
            .vars
            .own
            .total
            .get()
            .wrapping_add(time),
    );
    // Converted: `rc2k_core::net::players::raced`.
    crate::net::owner::net().vars.own.head.set(
        PlayerRecord::STATE,
        rc2k_core::net::players::raced(crate::net::owner::net().vars.own.state()),
    );
    crate::net::owner::net().vars.after_race.set(1);
    crate::net::list_players::list_players_regs()
}
