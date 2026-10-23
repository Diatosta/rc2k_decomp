//! `net::connection_entry` (0x418530): the record of connection eax (0x54 bytes each in the
//! table `dword_52AC38`, `dword_52AC34` of them): +0 its name (0x40 bytes), +0x40 its
//! service provider's GUID, +0x50 the connection's DirectPlay address.
//!
//! In:  eax = the connection's number
//! Out: eax = its record; the flags of the `add` (no caller reads them)
//! A number not below the count (unsigned) is fatal error 0x80A61010, no table fatal error
//! 0x80A61011. Callers: `net::keep_connection`, `net::find_saved_connection`,
//! `net::connection_found`, `net::build_address`, `net::connections_frame`. Shadow mode: both
//! run, eax compared. Self-test: a copy whose jumps to the fatal stubs land in the test,
//! against the Rust version, on numbers in and past the list and no table.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{Part, connection};

/// The Rust version: the registers it leaves.
pub fn connection_entry(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::net::lists::record` (in `net::lobby::record_of`).
    Ok(Regs {
        eax: connection(entry.eax)?,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        connection_entry(r, &mut crate::net::lobby::game_part)
    })
}
