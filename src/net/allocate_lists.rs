//! `net::allocate_lists` (0x418576): room for 0x100 connections (0x5400 bytes, to
//! `dword_52AC38`) and 0x100 sessions (0x6800 bytes, to `dword_52ACF8`) on the network's
//! memory list (`g_netMemory`, `memory::allocate`), and both counts 0.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! `memory::allocate` coming back with carry: fatal error 0x80E68021 for the first, 0x80E68022
//! for the second. Caller: `net::find_connections`.

use crate::errors::Refused;
use crate::net::store::NET;

/// The Rust version (`memory::allocate` refuses rather than come back with the carry, so
/// the fatal errors 0x80E68021 / 0x80E68022 are not reached).
pub fn allocate_lists() -> Result<(), Refused> {
    // Converted: `rc2k_core::net::lists::TABLES` (the tables Rust's, `net::tables`).
    crate::net::tables::take()?;
    NET.connections.set(0);
    NET.sessions.set(0);
    Ok(())
}
