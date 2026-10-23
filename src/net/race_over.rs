//! `net::race_over` (0x41732E): this player's race done: `dword_52AD78` and `dword_52AD7C` 0,
//! the race's time (eax) to the record (+8) and added to its total (+0xC), its state
//! "raced" (bits 0 and 1 off, bit 2 on), "after a race" (`dword_52AD80` 1), and the players
//! listed (`net::list_players_regs`: the record sent).
//!
//! In:  eax = the race's time
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `net::lets_rally`. Its call through `part`. Shadow mode runs the original alone.
//! Self-test: a copy whose call goes to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    ALL_RACED, BACK_FROM_RACE, OWN, OWN_TIME, OWN_TOTAL, Part, RACE_OVER, g, gb, lobby_entry, p, pb,
};

const LIST_PLAYERS: u32 = 0x417A03; // net::list_players_regs

/// The Rust version: the registers it leaves (the entry's).
pub fn race_over(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(ALL_RACED, 0);
    p(RACE_OVER, 0);
    p(OWN_TIME, r.eax);
    p(OWN_TOTAL, g(OWN_TOTAL).wrapping_add(r.eax));
    // Converted: `rc2k_core::net::players::raced`.
    pb(OWN, rc2k_core::net::players::raced(gb(OWN)));
    p(BACK_FROM_RACE, 1);
    part(LIST_PLAYERS, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, race_over)
}
