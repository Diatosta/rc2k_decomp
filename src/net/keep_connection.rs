//! `net::keep_connection` (0x417661): the connection kept as the one to use next time: its
//! service provider's GUID (record +0x40, 16 bytes) to `saved_connection`, its name (64 bytes)
//! to `saved_name`.
//!
//! Takes `connection`.
//! Callers: `net::start_link`, `net::settings_link`, `net::settings_done`. Refuses what
//! `net::lobby::connection` refuses (its fatal errors).

use crate::errors::Refused;
use crate::net::owner::net;
use crate::net::tables::with_connection;
pub fn keep_connection(number: u32) -> Result<(), Refused> {
    let net = net();
    let (provider, name) = with_connection(number, |c| (c.provider, c.name))?;
    net.vars
        .saved_connection
        .write_at(0, &provider, "the saved connection")?;
    net.vars
        .saved_name
        .write_at(0, &name, "the saved connection")?;
    Ok(())
}
