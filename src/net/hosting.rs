//! `net::hosting` (0x4176D5): "hosting" shown (string 0x151, as `net::connecting` shows
//! its), then the session made (`net::host_session`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags `net::host_session` leaves
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

const HOST_SESSION: u32 = 0x418B13; // net::host_session

/// The Rust version: the registers it leaves (the entry's, the call's flags).
pub fn hosting(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    status(&mut r, 0x151, part)?;
    part(HOST_SESSION, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, hosting)
}
