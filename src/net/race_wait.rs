//! `net::race_wait` (0x4171FE): one turn of waiting for the race to start: this player's
//! state made "waiting" (bit 1 off, bit 2 on), its time -1; once a second the players listed
//! (`net::list_players_regs`); on the host, with every player waiting (state bit 2, those
//! with 0x80 left out), the sync of `net::lobby_loop` with 'P' messages, and when all have
//! answered `dword_52AD7C` 1.
//!
//! In:  nothing
//! Out: eax = `dword_52AD7C` (1: go); the rest kept (`pusha` .. `popa`); carry set with no
//!      players on the host (or a failed listing, which never comes), else clear
//! Caller: `net::lets_rally`, once a frame. Its calls through `part`. Shadow mode runs the
//! original alone (it sends). Self-test: a copy whose calls go to stand-ins, against the
//! Rust version: not the host, the host waiting for one, all answered, no players.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    CLOCK_A, OWN, OWN_TIME, PLAYERS, Part, RACE_OVER, TICK, g, gb, lobby_entry, p, pb,
};
use crate::net::lobby_loop::{all_states, states_start, sync};
use rc2k_core::net::players;

const LIST_PLAYERS: u32 = 0x417A03; // net::list_players_regs

/// The Rust version: the registers it leaves.
pub fn race_wait(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs, carry: bool| {
        let mut l = Regs {
            eflags: r.eflags,
            ..*entry
        };
        l.set_carry(carry);
        l.eax = g(RACE_OVER);
        l
    };
    // Converted: `rc2k_core::net::players` (`waiting`, `all_in_state` in `all_states`).
    pb(OWN, players::waiting(gb(OWN)));
    p(OWN_TIME, u32::MAX);
    part(TICK, &mut r)?;
    r.ebx = r.eax;
    r.eax = r.eax.wrapping_sub(g(CLOCK_A));
    if (r.eax.wrapping_sub(1000) as i32) >= 0 {
        p(CLOCK_A, r.ebx);
        part(LIST_PLAYERS, &mut r)?;
        if r.carry() {
            return Ok(left(&r, true));
        }
    }
    if gb(OWN) & players::HOST != 0 {
        states_start(&mut r, players::WAITING);
        if g(PLAYERS) == 0 {
            return Ok(left(&r, true));
        }
        if all_states(&mut r, players::WAITING, g(PLAYERS)) == players::WAITING
            && sync(&mut r, b'P', part)?
        {
            p(RACE_OVER, 1);
        }
    }
    Ok(left(&r, false))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, race_wait)
}
