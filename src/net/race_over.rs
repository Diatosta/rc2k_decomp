//! `net::race_over` (0x41732E): this player's race done: `dword_52AD78` and `dword_52AD7C` 0,
//! the race's time (eax) to the record (+8) and added to its total (+0xC), its state
//! "raced" (bits 0 and 1 off, bit 2 on), "after a race" (`dword_52AD80` 1), and the players
//! listed (`net::list_players_regs`: the record sent).
//!
//! Takes `time`.
//! Caller: `net::lets_rally`.

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::net::store::PlayerRecord;
pub fn race_over(net: &Net, time: u32) -> Result<(), Refused> {
    net.vars.all_raced.set(0);
    net.vars.race_over.set(0);
    net.vars.own.race_time.set(time);
    net.vars
        .own
        .total
        .set(net.vars.own.total.get().wrapping_add(time));
    // Converted: `rc2k_core::net::players::raced`.
    net.vars.own.head.set(
        PlayerRecord::STATE,
        rc2k_core::net::players::raced(net.vars.own.state()),
    );
    net.vars.after_race.set(1);
    crate::net::list_players::list_players_regs(&net.vars)
}
