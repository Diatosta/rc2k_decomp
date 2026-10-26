//! `net::find_connections_regs` (0x4174BD): `net::find_connections` inside `pusha` .. `popa`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the carry `net::find_connections` leaves (set: no connection)
//! Callers: `net::start_link`, `net::settings_link`, `net::settings_done`. Its call through `part`. Shadow mode runs the original alone. Self-test: a copy whose call goes to a
//! stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{Part, lobby_entry, test_eax};

const CALLEE: u32 = 0x41867C; // net::find_connections

/// The Rust version: the registers it leaves (the entry's, the flags the call's).
pub fn find_connections_regs(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(CALLEE, &mut r)?;
    let _ = test_eax;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, find_connections_regs)
}
