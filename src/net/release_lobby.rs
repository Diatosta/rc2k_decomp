//! `net::release_lobby` (0x41900F): with a lobby object (`g_dplayLobby` not 0), the object
//! released (`IDirectPlayLobby2A` +8) and `g_dplayLobby` made 0.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`; the flags are read by no caller:
//!      `lq -s 0x41900F`)
//! Caller: `net::destroy_connection`. `Release` from Rust (`net::dplay`).
//! Refuses nothing.

use crate::net::dplay::{RELEASE, method};
use crate::net::lobby::{LOBBY, g, p};

/// The Rust version.
pub fn release_lobby() {
    if g(LOBBY) != 0 {
        unsafe { method(g(LOBBY), RELEASE, &[]) };
        p(LOBBY, 0);
    }
}
