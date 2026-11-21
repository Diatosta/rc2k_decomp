//! Ends the network race and, with a DirectPlay object, destroys the own player, closes the
//! session and releases both objects; the network's memory is given back either way. Refuses what
//! `net::free_messages` refuses.

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::net::{close_session, destroy_player, free_messages, release_dplay, release_lobby};
use crate::race::viewer_screen_store::{NetTexts, texts};
pub(super) fn destroy_connection(net: &Net, lines: &NetTexts) -> Result<(), Refused> {
    let link = &net.link;
    if link.has_dplay() {
        net.race_slot.end();
        crate::net::lobby::log_bytes(&lines.text(texts::NET_DESTROY_CONNECTION), true, 2);
        destroy_player::destroy_player(link);
        close_session::close_session(link);
        release_dplay::release_dplay(link, lines);
        release_lobby::release_lobby(link);
    }
    free_messages::free_messages(net)
}
