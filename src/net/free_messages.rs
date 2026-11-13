//! Gives the network's memory back and zeroes the four connection and session counts and
//! tables. Refuses what `memory::free_list` refuses.

use crate::errors::Refused;
use crate::net::store::NetShared;
pub fn free_messages(vars: &NetShared) -> Result<(), Refused> {
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
