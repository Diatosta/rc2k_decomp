//! `net::disable_new_players` (0x418D8A): on the host (state bit 0x10 of `own`), "crate::net::owner::NET_OWNER.vars:
//! Disable new players", and with DirectPlay's object there the session's description
//! (`session_desc`) given DPSESSION_NEWPLAYERSDISABLED and DPSESSION_JOINDISABLED (0x21) and
//! set (`SetSessionDesc`, flags 0); its result logged.
//!
//! Caller: `net::close_if_host`.

use crate::net::lobby::{log, log_hex};
use crate::net::owner::Net;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

const CLOSED: u32 = 0x21; // DPSESSION_NEWPLAYERSDISABLED | DPSESSION_JOINDISABLED
pub fn disable_new_players(net: &Net) {
    if net.vars.own.state() & 0x10 == 0 {
        return;
    }
    log(
        VIEWER_PANEL.text_ptr(texts::NET_DISABLE_NEW_PLAYERS),
        false,
        2,
    );
    if net.vars.dplay.get() == 0 {
        return;
    }
    net.vars
        .session_desc
        .flags
        .set(net.vars.session_desc.flags.get() | CLOSED);
    let desc =
        crate::net::dp_objects::desc(&net.vars.session_desc, &net.setup().machine_name.read());
    let result = crate::net::dp_objects::set_session_desc(net.vars.dplay.get(), &desc);
    log_hex(result, true);
}

/// Not the original's: on the host, with the session closed by
/// `disable_new_players`, the session opened to new players again (0x21 cleared), so a player
/// who left can join again between races. The original closed it for good at the first race.
/// Caller: `net::lobby_loop`, each frame no player is still racing (only then: a player let
/// in while others raced would be sent their race's messages).
pub fn enable_new_players(net: &Net) {
    let flags = net.vars.session_desc.flags.get();
    if net.vars.own.state() & 0x10 == 0 || flags & CLOSED == 0 || net.vars.dplay.get() == 0 {
        return;
    }
    net.vars.session_desc.flags.set(flags & !CLOSED);
    let desc =
        crate::net::dp_objects::desc(&net.vars.session_desc, &net.setup().machine_name.read());
    let result = crate::net::dp_objects::set_session_desc(net.vars.dplay.get(), &desc);
    crate::framework::log(&format!("net: new players let in again ({result:#x})"));
}
