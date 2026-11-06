//! `net::stopping` (0x417089): with the session on (`g_netSession` 1), the background
//! (`net::show_background`) and "stopping" shown (string 0x157, as `net::connecting` shows
//! its); then the network stopped (`net::stop`).
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! Caller: `net::start_link`.

use crate::errors::Refused;
use crate::net::owner::net;

/// The Rust version.
pub fn stopping() -> Result<(), Refused> {
    if net().vars.session.get() == 1 {
        crate::net::show_background::show_background()?;
        crate::net::connecting::status(0x157)?;
    }
    crate::net::stop::stop()
}
