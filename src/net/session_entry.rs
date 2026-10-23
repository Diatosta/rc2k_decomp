//! `net::session_entry` (0x418553): the record of session eax (0x68 bytes each in the table
//! `dword_52ACF8`, `dword_52ACF4` of them): +0 its name (0x40 bytes), +0x40 its instance
//! GUID, +0x50 the most players, +0x54 the players in it, +0x58 its password GUID (a
//! reading of what `net::session_found` copies from the session's description).
//!
//! In:  eax = the session's number
//! Out: eax = its record; the flags of the `add` (no caller reads them)
//! A number not below the count (unsigned) is fatal error 0x80A61030, no table fatal error
//! 0x80A61031. Callers: `net::choose_session`, `net::sessions_frame`, `net::session_found`,
//! `net::join_session`. Shadow mode: both run, eax compared. Self-test: as
//! `net::connection_entry`'s.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{Part, session};

/// The Rust version: the registers it leaves.
pub fn session_entry(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::net::lists::record` (in `net::lobby::record_of`).
    Ok(Regs {
        eax: session(entry.eax)?,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| session_entry(r, &mut crate::net::lobby::game_part))
}
