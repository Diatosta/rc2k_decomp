//! `net::free_messages` (0x4185BB): the network's memory given back (`memory::free_list`
//! on `stru_52A928`) and four of its counts (`dword_52AC34`, `dword_52AC38`,
//! `dword_52ACF4`, `dword_52ACF8`) made 0.
//!
//!
//! See `net/mod.rs`. One caller, `net::destroy_connection`. Refuses what
//! `memory::free_list` refuses.

use crate::errors::Refused;
use crate::net::owner::net;
pub fn free_messages() -> Result<(), Refused> {
    let vars = &net().vars;
    { crate::memory::free_list::free_list(&vars.memory) }?;
    crate::net::tables::NET_TABLES.give_back();
    for cell in [
        &vars.connections,
        &vars.connection_table,
        &vars.sessions,
        &vars.session_table,
    ] {
        cell.set(0);
    }
    Ok(())
}
