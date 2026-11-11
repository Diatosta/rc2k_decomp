//! `net::close_if_host` (0x41837B): on the host (state bit 0x10 of `own`), no more
//! players let in (`net::disable_new_players`).
//!
//! Caller: `net::start_link`, before each race.

use crate::net::owner::Net;
pub fn close_if_host(net: &Net) {
    // Converted: `rc2k_core::net::record::is_host`.
    if rc2k_core::net::record::is_host(net.vars.own.state()) {
        crate::net::disable_new_players::disable_new_players(net);
    }
}
