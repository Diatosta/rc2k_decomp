//! `net::settings_link` (0x416EEC): Enter on the settings screen's connection: the
//! connections listed (`net::find_connections_regs`), one chosen (`net::choose_connection`)
//! and kept (`net::keep_connection`); then the network stopped again (`net::stop`).
//!
//! Caller: `net::settings_loop`. Long-running (a list screen).

use crate::errors::Refused;
use crate::race::viewer_screen_store::NetTexts;
pub fn settings_link(o: crate::owners::Owners<'static>, lines: &NetTexts) -> Result<(), Refused> {
    let net = o.net;
    if crate::net::find_connections::find_connections(&net.vars, o.strings, lines)? {
        let chosen = crate::net::choose_connection::choose_connection(o)?;
        if (chosen as i32) >= 0 {
            crate::net::keep_connection::keep_connection(net, chosen)?;
        }
    }
    crate::net::stop::stop(o, &net.vars, lines)
}
