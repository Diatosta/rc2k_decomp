//! `net::dplay_send` (0x4196FF): DirectPlay's `Send` (`IDirectPlay4A` +0x68) from this
//! machine's player (`g_netPlayerId`) through `g_dplay`.
//!
//! In:  `to` (eax: a player or the group), `flags` (edx: DPSEND_*), `data` (ebx: the
//!      bytes), `size` (ecx: how many)
//! Out: nothing (the original keeps every register; its flags, carry set on a failure,
//!      are read by no caller: `lq -s 0x4196FF`)
//! Six callers (`net::send_message`, `net::send_chat`, `net::race_wait`, `net::lobby_loop`,
//! `net::take_messages` twice). `Send` from Rust (`net::dplay`, section 15). Refuses nothing
//! (no check of `g_dplay`, as the original).

use crate::net::dplay::{SEND, method};
use crate::net::lobby::{DPLAY, PLAYER_ID, g};

/// The Rust version.
pub fn dplay_send(to: u32, flags: u32, data: u32, size: u32) {
    unsafe { method(g(DPLAY), SEND, &[g(PLAYER_ID), to, flags, data, size]) };
}
