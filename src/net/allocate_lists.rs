//! `net::allocate_lists` (0x418576): room for 0x100 connections (0x5400 bytes, to
//! `dword_52AC38`) and 0x100 sessions (0x6800 bytes, to `dword_52ACF8`) on the network's
//! memory list (`g_netMemory`, `memory::allocate`), and both counts 0.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! `memory::allocate` coming back with carry: fatal error 0x80E68021 for the first, 0x80E68022
//! for the second. Caller: `net::find_connections`.

use crate::errors::Refused;
use crate::net::lobby::{CONNECTION_TABLE, CONNECTIONS, SESSION_TABLE, SESSIONS, net_allocate, p};
use rc2k_core::net::lists;

/// The Rust version (`memory::allocate` refuses rather than come back with the carry, so
/// the fatal errors 0x80E68021 / 0x80E68022 are not reached).
pub fn allocate_lists() -> Result<(), Refused> {
    // Converted: `rc2k_core::net::lists::TABLES`.
    for ((bytes, _code), table) in lists::TABLES
        .into_iter()
        .zip([CONNECTION_TABLE, SESSION_TABLE])
    {
        p(table, net_allocate(bytes)?);
    }
    p(CONNECTIONS, 0);
    p(SESSIONS, 0);
    Ok(())
}
