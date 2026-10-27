//! `net::free_messages` (0x4185BB): the network's memory given back (`memory::free_list`
//! on `stru_52A928`) and four of its counts (`dword_52AC34`, `dword_52AC38`,
//! `dword_52ACF4`, `dword_52ACF8`) made 0.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`)
//!
//! See `net/mod.rs`. One caller, `net::destroy_connection`. Refuses what
//! `memory::free_list` refuses.

use crate::errors::Refused;
use crate::net::store::NET;

/// The Rust version.
pub fn free_messages() -> Result<(), Refused> {
    { crate::memory::free_list::free_list(NET.memory.as_ptr() as *mut _) }?;
    crate::net::tables::NET_TABLES.give_back();
    crate::net::lobby::give_back_lobby_settings();
    for cell in [
        &NET.connections,
        &NET.connection_table,
        &NET.sessions,
        &NET.session_table,
    ] {
        cell.set(0);
    }
    Ok(())
}
