//! `net::close_if_host` (0x41837B): on the host (state bit 0x10 of `byte_52AD64`), no more
//! players let in (`net::disable_new_players`).
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`; the flags read by no caller)
//! Caller: `net::start_link`, before each race.

use crate::net::owner::net;

/// The Rust version.
pub fn close_if_host() {
    // Converted: `rc2k_core::net::record::is_host`.
    if rc2k_core::net::record::is_host(net().vars.own.state()) {
        crate::net::disable_new_players::disable_new_players();
    }
}
