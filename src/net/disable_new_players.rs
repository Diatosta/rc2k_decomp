//! On the host, with a DirectPlay object, sets the session description to refuse new players
//! and joins ("NET: Disable new players").

use crate::net::lobby::{log_bytes, log_hex};
use crate::net::owner::Net;
use crate::race::viewer_screen_store::{NetTexts, texts};

const CLOSED: u32 = 0x21; // DPSESSION_NEWPLAYERSDISABLED | DPSESSION_JOINDISABLED
pub fn disable_new_players(net: &Net, lines: &NetTexts) {
    if !net.vars.own.state().is_host() {
        return;
    }
    log_bytes(&lines.text(texts::NET_DISABLE_NEW_PLAYERS), false, 2);
    if !net.link.has_dplay() {
        return;
    }
    net.link
        .session_desc()
        .flags
        .set(net.link.session_desc().flags.get() | CLOSED);
    let desc =
        crate::net::dp_objects::desc(net.link.session_desc(), &net.setup().machine_name.read());
    let result = crate::net::dp_objects::set_session_desc(net.link.dplay(), &desc);
    log_hex(result, true);
}

/// On the host, with the session closed by `disable_new_players`, the session is opened to
/// new players again (0x21 cleared), so a player who left can join again between races.
/// Caller: `net::lobby_loop`, each frame no player is still racing (only then: a player let
/// in while others raced would be sent their race's messages).
pub fn enable_new_players(net: &Net) {
    let flags = net.link.session_desc().flags.get();
    if !net.vars.own.state().is_host() || flags & CLOSED == 0 || !net.link.has_dplay() {
        return;
    }
    net.link.session_desc().flags.set(flags & !CLOSED);
    let desc =
        crate::net::dp_objects::desc(net.link.session_desc(), &net.setup().machine_name.read());
    let result = crate::net::dp_objects::set_session_desc(net.link.dplay(), &desc);
    crate::program_log::log(&format!("net: new players let in again ({result:#x})"));
}
