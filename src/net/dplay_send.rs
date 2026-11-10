//! `net::dplay_send` (0x4196FF): DirectPlay's `Send` (`IDirectPlay4A` +0x68) from this
//! machine's player (`g_netPlayerId`) through `g_dplay`.
//!
//! Takes `to`, `flags`, `data` and `size`. The original sets the carry on a failure; no caller
//! reads it (`lq -s 0x4196FF`).
//! Six callers (`net::send_message`, `net::send_chat`, `net::race_wait`, `net::lobby_loop`,
//! `net::take_messages` twice). `Send` from Rust (`net::dplay`). Refuses nothing
//! (no check of `g_dplay`, as the original).

use crate::net::store::NetVars;
pub fn dplay_send(vars: &NetVars, to: u32, data: &[u8]) {
    crate::net::dp_objects::send(vars.dplay.get(), vars.player_id.get(), to, data);
}
