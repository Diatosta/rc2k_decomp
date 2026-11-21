//! Enter on the settings screen's connection: lists the connections, lets one be chosen and
//! keeps it; then stops the network again. A list screen, so it runs long.

use crate::errors::Refused;
use crate::race::viewer_screen_store::NetTexts;
pub(super) fn settings_link(
    o: crate::owners::Owners<'static>,
    lines: &NetTexts,
) -> Result<(), Refused> {
    let net = o.net;
    if crate::net::find_connections::find_connections(net, o.strings, lines)? {
        let chosen = crate::net::choose_connection::choose_connection(o)?;
        if (chosen as i32) >= 0 {
            crate::net::keep_connection::keep_connection(net, chosen)?;
        }
    }
    crate::net::stop::stop(o, net, lines)
}
