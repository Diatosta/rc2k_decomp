//! `net::joining` (0x4179B6): "joining" shown (string 0x153, as `net::connecting` shows
//! its), then session eax joined (`net::join_session`).
//!
//! In:  eax = the session's number
//! Out: every register kept (`pusha` .. `popa`); the flags `net::join_session` leaves
//!      (carry: it failed)
//! Caller: `net::start_link`. Its calls through `part` (the frames through the game).
//! Shadow mode runs the original alone. Self-test: as `net::connecting`'s.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::connecting::status;
use crate::net::lobby::{Part, lobby_entry};

const JOIN_SESSION: u32 = 0x418C6D; // net::join_session

/// The Rust version: the registers it leaves (the entry's, the call's flags).
pub fn joining(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.esi = r.eax;
    status(&mut r, 0x153, part)?;
    r.eax = r.esi;
    part(JOIN_SESSION, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, joining)
}
