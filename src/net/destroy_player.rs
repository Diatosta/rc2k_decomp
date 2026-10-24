//! `net::destroy_player` (0x41916B): DirectPlay's `DestroyPlayer` (`IDirectPlay4A` +0x24)
//! on `g_netPlayerId` through `g_dplay`, and the id made 0 — but only when it is 0 already
//! (`cmp g_netPlayerId, 0; jnz` past the call: the game's slip, section 16; this machine's
//! player is never destroyed, `DestroyPlayer(0)` is called instead).
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`; the flags are read by no caller:
//!      `lq -s 0x41916B`)
//! Caller: `net::destroy_connection`. `DestroyPlayer` from Rust (`net::dplay`).
//! Refuses nothing.

use crate::net::dplay::{DESTROY_PLAYER, method};
use crate::net::lobby::{DPLAY, PLAYER_ID, g, p};

/// The Rust version.
pub fn destroy_player() {
    let id = g(PLAYER_ID);
    if id == 0 {
        unsafe { method(g(DPLAY), DESTROY_PLAYER, &[id]) };
        p(PLAYER_ID, 0);
    }
}
