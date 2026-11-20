//! On the host, lets no more players in (`net::disable_new_players`).

use crate::net::owner::Net;
pub(super) fn close_if_host(net: &Net, lines: &crate::race::viewer_screen_store::NetTexts) {
    if net.vars.own.state().is_host() {
        crate::net::disable_new_players::disable_new_players(net, lines);
    }
}
