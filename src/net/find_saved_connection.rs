//! `net::find_saved_connection` (0x4185DF): the connection listed whose service provider's
//! GUID (record +0x40) is the one saved (`saved_connection`, `system::same_guid`).
//!
//! Returns the connection's number; None when none is listed or none matches.
//! Callers: `net::start_link`, `net::settings_done`.

use crate::errors::Refused;
use crate::net::owner::net;
use crate::net::tables::with_connection;
use rc2k_core::net::lists;
pub fn find_saved_connection() -> Result<Option<u32>, Refused> {
    let vars = &net().vars;
    // Converted: `rc2k_core::net::lists::find_listed`.
    let saved = net().address().saved_connection.read();
    lists::find_listed(vars.connections.get(), |n| -> Result<bool, Refused> {
        // `system::same_guid`: the four dwords alike.
        with_connection(n, |c| c.provider == saved)
    })
}
