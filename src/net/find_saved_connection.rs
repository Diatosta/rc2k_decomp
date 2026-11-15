//! The listed connection whose service provider GUID is the saved one. Returns its number,
//! or None when none is listed or none matches.

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::net::tables::with_connection;
use rc2k_core::net::lists;
pub fn find_saved_connection(net: &Net) -> Result<Option<u32>, Refused> {
    // The logic is in `rc2k_core::net::lists::find_listed`.
    let saved = net.setup().saved_connection.read();
    lists::find_listed(net.link.connections().get(), |n| -> Result<bool, Refused> {
        // `system::same_guid`: the four dwords alike.
        with_connection(&net.link, n, |c| c.provider == saved)
    })
}
