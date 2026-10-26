//! `net::release_lobby` (0x41900F): with a lobby object (`g_dplayLobby` not 0), the object
//! released (`IDirectPlayLobby2A` +8) and `g_dplayLobby` made 0.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`; the flags are read by no caller:
//!      `lq -s 0x41900F`)
//! Caller: `net::destroy_connection`. `Release` from Rust (`net::dplay`).
//! Refuses nothing.

use crate::net::store::NET;

/// The Rust version.
pub fn release_lobby() {
    if NET.lobby.get() != 0 {
        crate::net::dp_objects::release(NET.lobby.get());
        NET.lobby.set(0);
    }
}
