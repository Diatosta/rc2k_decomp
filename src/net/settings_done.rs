//! `net::settings_done` (0x416F0B): the connection saved checked: the connections listed
//! (`net::find_connections_regs`) and the saved one among them
//! (`net::find_saved_connection`) kept again (`net::keep_connection`, its name fresh);
//! if either fails the saved connection forgotten (`unk_52AC94` 16 bytes, `unk_52ACA4` 64,
//! `system::zero_memory`); then the network stopped (`net::stop`).
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! Caller: `net::settings_loop` (on entering it).

use crate::errors::Refused;
use crate::net::lobby::zero;
use crate::net::store::NetVars;

/// The Rust version.
pub fn settings_done(vars: &'static NetVars) -> Result<(), Refused> {
    let saved = if crate::net::find_connections::find_connections(vars)? {
        crate::net::find_saved_connection::find_saved_connection()?
    } else {
        None
    };
    match saved {
        Some(n) => crate::net::keep_connection::keep_connection(n)?,
        None => {
            zero(&vars.saved_connection, 0x10)?;
            zero(&vars.saved_name, 0x40)?;
        }
    }
    crate::net::stop::stop(vars)
}
