//! Keeps the connection as the one to use next time: its service provider GUID and its name.

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::net::tables::with_connection;
pub(super) fn keep_connection(net: &Net, number: u32) -> Result<(), Refused> {
    let (provider, name) = with_connection(&net.link, number, |c| (c.provider, c.name))?;
    net.setup()
        .saved_connection
        .write_at(0, &provider, "the saved connection")?;
    net.settings()
        .saved_name
        .write_at(0, &name, "the saved connection")?;
    Ok(())
}
