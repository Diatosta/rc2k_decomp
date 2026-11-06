//! `net::settings_link` (0x416EEC): Enter on the settings screen's connection: the
//! connections listed (`net::find_connections_regs`), one chosen (`net::choose_connection`)
//! and kept (`net::keep_connection`); then the network stopped again (`net::stop`).
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! Caller: `net::settings_loop`. Long-running (a list screen).

use crate::errors::Refused;
use crate::net::store::NetVars;

/// The Rust version.
pub fn settings_link(vars: &NetVars) -> Result<(), Refused> {
    if crate::net::find_connections::find_connections(vars)? {
        let chosen = crate::net::choose_connection::choose_connection(vars)?;
        if (chosen as i32) >= 0 {
            crate::net::keep_connection::keep_connection(chosen)?;
        }
    }
    crate::net::stop::stop(vars)
}
