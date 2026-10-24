//! `net::close_session` (0x418DD3): with a session open (`dword_52ACFC` 1), the switch made
//! 0 and DirectPlay's `Close` (`IDirectPlay4A` +0x10) called through `g_dplay`.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`; the flags are read by no caller:
//!      `lq -s 0x418DD3`)
//! Caller: `net::destroy_connection`. `Close` from Rust (`net::dplay`). Refuses
//! nothing.

use crate::net::dplay::{CLOSE, method};
use crate::net::lobby::{DPLAY, SESSION_OPEN, g, p};

/// The Rust version.
pub fn close_session() {
    if g(SESSION_OPEN) == 1 {
        p(SESSION_OPEN, 0);
        unsafe { method(g(DPLAY), CLOSE, &[]) };
    }
}
