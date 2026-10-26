//! `net::close_if_host` (0x41837B): on the host (state bit 0x10 of `byte_52AD64`), no more
//! players let in (`net::disable_new_players`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags of the test, or the call's (no
//!      caller reads them)
//! Caller: `net::start_link`, before each race. Its call through `part`. Shadow mode runs
//! the original alone. Self-test: a copy whose call goes to a stand-in, against the Rust
//! version, host or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{OWN, Part, gb, lobby_entry};

const DISABLE_NEW_PLAYERS: u32 = 0x418D8A; // net::disable_new_players

/// The Rust version: the registers it leaves (the entry's).
pub fn close_if_host(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::net::record::is_host`.
    if rc2k_core::net::record::is_host(gb(OWN)) {
        part(DISABLE_NEW_PLAYERS, &mut r)?;
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, close_if_host)
}
