//! `net::close_session` (0x418DD3): with a session open (`dword_52ACFC` 1), the switch made
//! 0 and DirectPlay's `Close` (`IDirectPlay4A` +0x10) called through `g_dplay`.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`; the flags are read by no caller:
//!      `lq -s 0x418DD3`)
//! Caller: `net::destroy_connection`. `Close` from Rust (`net::dplay`). Refuses
//! nothing.

use crate::net::store::NetVars;

/// The Rust version.
pub fn close_session(vars: &NetVars) {
    if vars.session_open.get() == 1 {
        vars.session_open.set(0);
        crate::net::dp_objects::close(vars.dplay.get());
    }
}
