//! `net::destroy_player` (0x41916B): DirectPlay's `DestroyPlayer` (`IDirectPlay4A` +0x24)
//! on `g_netPlayerId` through `g_dplay`, and the id made 0 — but only when it is 0 already
//! (`cmp g_netPlayerId, 0; jnz` past the call: the game's slip, section 16; this machine's
//! player is never destroyed, `DestroyPlayer(0)` is called instead).
//!
//! Caller: `net::destroy_connection`. `DestroyPlayer` from Rust (`net::dplay`).
//! Refuses nothing.

use crate::net::store::NetVars;

/// The Rust version.
pub fn destroy_player(vars: &NetVars) {
    let id = vars.player_id.get();
    if id == 0 {
        crate::net::dp_objects::destroy_player(vars.dplay.get(), id);
        vars.player_id.set(0);
    }
}
