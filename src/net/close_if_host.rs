//! On the host, lets no more players in (`net::disable_new_players`).

use crate::net::owner::Net;
pub fn close_if_host(net: &Net, lines: &crate::race::viewer_screen_store::NetTexts) {
    // The logic is in `rc2k_core::net::record::is_host`.
    if rc2k_core::net::record::is_host(net.vars.own.state()) {
        crate::net::disable_new_players::disable_new_players(net, lines);
    }
}
