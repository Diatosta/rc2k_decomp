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

/// The Rust version.
pub fn free_messages() -> Result<(), Refused> {
    {
        crate::memory::free_list::free_list(crate::pointers::GamePtr::to(
            &crate::net::owner::net().vars.memory,
        ))
    }?;
    crate::net::tables::NET_TABLES.give_back();
    for cell in [
        &crate::net::owner::net().vars.connections,
        &crate::net::owner::net().vars.connection_table,
        &crate::net::owner::net().vars.sessions,
        &crate::net::owner::net().vars.session_table,
    ] {
        cell.set(0);
    }
    Ok(())
}
