//! `net::release_lobby` (0x41900F): with a lobby object (`g_dplayLobby` not 0), the object
//! released (`IDirectPlayLobby2A` +8) and `g_dplayLobby` made 0.
//!
//! Caller: `net::destroy_connection`. `Release` from Rust (`net::dplay`).
//! Refuses nothing.

use crate::net::store::NetVars;

/// The Rust version.
pub fn release_lobby(vars: &NetVars) {
    if vars.lobby.get() != 0 {
        crate::net::dp_objects::release(vars.lobby.get());
        vars.lobby.set(0);
    }
}
