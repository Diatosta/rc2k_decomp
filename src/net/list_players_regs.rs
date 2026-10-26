//! `net::list_players_regs` (0x417A03): `net::list_players`, then `test eax, eax` on the eax it leaves (its own, since it
//! keeps every register), inside `pusha` .. `popa`: the carry it gives is lost, so its
//! callers' `jb` after it are never taken (section 16).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags of `test eax, eax` (carry clear)
//! Callers: `net::lets_rally`, `net::race_wait`, `net::race_over`, `net::lobby_loop`. Its call through `part`. Shadow mode runs the original alone. Self-test: a copy whose call goes to a
//! stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{Part, lobby_entry, test_eax};

const CALLEE: u32 = 0x419192; // net::list_players

/// The Rust version: the registers it leaves (the entry's, the flags of `test eax, eax`).
pub fn list_players_regs(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(CALLEE, &mut r)?;
    test_eax(&mut r);
    let _ = test_eax;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, list_players_regs)
}
