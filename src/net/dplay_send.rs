//! `net::dplay_send` (0x4196FF): DirectPlay's `Send` (`IDirectPlay4A` +0x68) from this
//! machine's player (`g_netPlayerId`) through `g_dplay`.
//!
//! In:  `to` (eax: a player or the group), `flags` (edx: DPSEND_*), `data` (ebx: the
//!      bytes), `size` (ecx: how many)
//! Out: nothing (the original keeps every register; its flags, carry set on a failure,
//!      are read by no caller: `lq -s 0x4196FF`)
//! Six callers (`net::send_message`, `net::send_chat`, `net::race_wait`, `net::lobby_loop`,
//! `net::take_messages` twice). `Send` from Rust (`net::dplay`). Refuses nothing
//! (no check of `g_dplay`, as the original).

use crate::net::store::NetVars;

/// The Rust version.
pub fn dplay_send(vars: &NetVars, to: u32, data: &[u8]) {
    crate::net::dp_objects::send(vars.dplay.get(), vars.player_id.get(), to, data);
}
