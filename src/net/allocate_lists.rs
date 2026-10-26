//! `net::allocate_lists` (0x418576): room for 0x100 connections (0x5400 bytes, to
//! `dword_52AC38`) and 0x100 sessions (0x6800 bytes, to `dword_52ACF8`) on the network's
//! memory list (`g_netMemory`, `memory::allocate`), and both counts 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! `memory::allocate` coming back with carry: fatal error 0x80E68021 for the first, 0x80E68022
//! for the second. Caller: `net::find_connections`. Its calls through `part`
//! (`net::lobby_calls`: the Rust version). Shadow mode runs the original alone (memory is
//! taken). Self-test: a copy whose calls go to a stand-in and whose jumps to the fatal stubs
//! land in the test, against the Rust version: both taken, either failing.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    ALLOCATE, CONNECTION_TABLE, CONNECTIONS, NET_MEMORY, Part, SESSION_TABLE, SESSIONS, a, fatal,
    lobby_entry, p,
};
use rc2k_core::net::lists;

/// The Rust version: the registers it leaves (the entry's).
pub fn allocate_lists(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::net::lists::TABLES`.
    for ((bytes, code), table) in lists::TABLES
        .into_iter()
        .zip([CONNECTION_TABLE, SESSION_TABLE])
    {
        r.eax = bytes;
        r.ebx = a(NET_MEMORY);
        part(ALLOCATE, &mut r)?;
        if r.carry() {
            return Err(fatal(code).into());
        }
        p(table, r.eax);
    }
    p(CONNECTIONS, 0);
    p(SESSIONS, 0);
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, allocate_lists)
}
