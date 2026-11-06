//! `net::disable_new_players` (0x418D8A): on the host (state bit 0x10 of `byte_52AD64`), "crate::net::owner::NET_OWNER.vars:
//! Disable new players", and with DirectPlay's object there the session's description
//! (`dword_5AB2AC`) given DPSESSION_NEWPLAYERSDISABLED and DPSESSION_JOINDISABLED (0x21) and
//! set (`SetSessionDesc`, flags 0); its result logged.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`; the carry it sets when not the
//!      host, with no object or on a failure is read by no caller: `lq -s 0x418D8A`)
//! Caller: `net::close_if_host`.

use crate::net::lobby::{log, log_hex};
use crate::net::owner::net;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

const CLOSED: u32 = 0x21; // DPSESSION_NEWPLAYERSDISABLED | DPSESSION_JOINDISABLED

/// The Rust version.
pub fn disable_new_players() {
    let net = net();
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
    let desc = crate::net::dp_objects::desc(&net.vars.session_desc, &net.vars.machine_name.read());
    let result = crate::net::dp_objects::set_session_desc(net.vars.dplay.get(), &desc);
    log_hex(result, true);
}
