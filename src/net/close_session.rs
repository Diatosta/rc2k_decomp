//! `net::close_session` (0x418DD3): with a session open (`session_open` 1), the switch made
//! 0 and DirectPlay's `Close` (`IDirectPlay4A` +0x10) called through `g_dplay`.
//!
//! Caller: `net::destroy_connection`. `Close` from Rust (`net::dplay`). Refuses
//! nothing.

use crate::net::store::NetVars;
pub fn close_session(vars: &NetVars) {
    if vars.session_open.get() == 1 {
        vars.session_open.set(0);
        crate::net::dp_objects::close(vars.dplay.get());
    }
}
