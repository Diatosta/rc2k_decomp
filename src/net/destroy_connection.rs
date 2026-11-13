//! Ends the network race and, with a DirectPlay object, destroys the own player, closes the
//! session and releases both objects; the network's memory is given back either way. Refuses what
//! `net::free_messages` refuses.

use crate::errors::Refused;
use crate::net::store::NetShared;
use crate::net::{close_session, destroy_player, free_messages, release_dplay, release_lobby};
use crate::race::viewer_screen_store::{NetTexts, texts};
pub fn destroy_connection(vars: &NetShared, lines: &NetTexts) -> Result<(), Refused> {
    if vars.dplay.get() != 0 {
        vars.session.set(0);
        crate::net::lobby::log_bytes(&lines.text(texts::NET_DESTROY_CONNECTION), true, 2);
        destroy_player::destroy_player(vars);
        close_session::close_session(vars);
        release_dplay::release_dplay(vars, lines);
        release_lobby::release_lobby(vars);
    }
    free_messages::free_messages(vars)
}
