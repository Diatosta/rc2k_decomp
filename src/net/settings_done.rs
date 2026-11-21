//! Checks the saved connection: the connections are listed and the saved one kept again if it
//! is among them, else forgotten; then the network is stopped.

use crate::errors::Refused;
use crate::net::lobby::zero;
use crate::race::viewer_screen_store::NetTexts;
pub(super) fn settings_done(
    o: crate::owners::Owners<'static>,
    lines: &NetTexts,
) -> Result<(), Refused> {
    let (net, strings) = (o.net, o.strings);
    let saved = if crate::net::find_connections::find_connections(net, strings, lines)? {
        crate::net::find_saved_connection::find_saved_connection(net)?
    } else {
        None
    };
    match saved {
        Some(n) => crate::net::keep_connection::keep_connection(net, n)?,
        None => {
            zero(&net.setup().saved_connection, 0x10)?;
            zero(&net.settings().saved_name, 0x40)?;
        }
    }
    crate::net::stop::stop(o, net, lines)
}
