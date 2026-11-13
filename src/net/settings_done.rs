//! `net::settings_done` (0x416F0B): the connection saved checked: the connections listed
//! (`net::find_connections_regs`) and the saved one among them
//! (`net::find_saved_connection`) kept again (`net::keep_connection`, its name fresh);
//! if either fails the saved connection forgotten (`saved_connection` 16 bytes, `saved_name` 64,
//! `system::zero_memory`); then the network stopped (`net::stop`).
//!
//! Caller: `net::settings_loop` (on entering it).

use crate::errors::Refused;
use crate::net::lobby::zero;
use crate::race::viewer_screen_store::NetTexts;
pub fn settings_done(o: crate::owners::Owners<'static>, lines: &NetTexts) -> Result<(), Refused> {
    let (net, strings) = (o.net, o.strings);
    let vars = &net.vars;
    let saved = if crate::net::find_connections::find_connections(vars, strings, lines)? {
        crate::net::find_saved_connection::find_saved_connection(net)?
    } else {
        None
    };
    match saved {
        Some(n) => crate::net::keep_connection::keep_connection(net, n)?,
        None => {
            zero(&net.address().saved_connection, 0x10)?;
            zero(&net.address().saved_name, 0x40)?;
        }
    }
    crate::net::stop::stop(o, vars, lines)
}
