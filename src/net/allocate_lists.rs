//! Room for 0x100 connections and 0x100 sessions on the network's memory list; both counts
//! start at 0. An allocation failure would be fatal error 0x80E68021 (connections) or
//! 0x80E68022 (sessions).

use crate::errors::Refused;
use crate::net::store::NetShared;

/// The fatal errors are not reached: `memory::allocate` refuses rather than fail.
pub fn allocate_lists(vars: &NetShared) -> Result<(), Refused> {
    // The logic is in `rc2k_core::net::lists::TABLES` (the tables in `net::tables`).
    crate::net::tables::take(vars)?;
    vars.connections.set(0);
    vars.sessions.set(0);
    Ok(())
}
