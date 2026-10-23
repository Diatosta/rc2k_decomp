//! `net::keep_connection` (0x417661): connection eax kept as the one to use next time: its
//! service provider's GUID (record +0x40, 16 bytes) to `unk_52AC94`, its name (64 bytes)
//! to `unk_52ACA4`.
//!
//! In:  eax = the connection's number
//! Out: every register kept (`pusha` .. `popa`)
//! Callers: `net::start_link`, `net::settings_link`, `net::settings_done`. Its call through
//! `part` (`net::connection_entry`, which may end in its fatal errors). Shadow mode runs the
//! original alone. Self-test: a copy whose call goes to a stand-in (a test record), against
//! the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::field;
use crate::framework::Regs;
use crate::net::lobby::{Part, SAVED_CONNECTION, SAVED_NAME, a, lobby_entry, movsb};
use crate::net::types::NetConnection;

const CONNECTION_ENTRY: u32 = 0x418530; // net::connection_entry

/// The Rust version: the registers it leaves (the entry's).
pub fn keep_connection(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(CONNECTION_ENTRY, &mut r)?;
    movsb(
        a(SAVED_CONNECTION),
        r.eax + field!(NetConnection, service_provider),
        0x10,
    );
    movsb(a(SAVED_NAME), r.eax, 0x40);
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, keep_connection)
}
