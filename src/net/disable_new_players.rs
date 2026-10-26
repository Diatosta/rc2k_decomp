//! `net::disable_new_players` (0x418D8A): on the host (state bit 0x10 of `byte_52AD64`), "NET:
//! Disable new players", and with DirectPlay's object there the session's description
//! (`dword_5AB2AC`) given DPSESSION_NEWPLAYERSDISABLED and DPSESSION_JOINDISABLED (0x21) and
//! set (`SetSessionDesc`, flags 0); its result logged.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`; the carry it sets when not the
//!      host, with no object or on a failure is read by no caller: `lq -s 0x418D8A`)
//! Caller: `net::close_if_host`.

use crate::net::lobby::{log, log_hex};
use crate::net::store::NET;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

const CLOSED: u32 = 0x21; // DPSESSION_NEWPLAYERSDISABLED | DPSESSION_JOINDISABLED

/// The Rust version.
pub fn disable_new_players() {
    if NET.own.state() & 0x10 == 0 {
        return;
    }
    log(
        VIEWER_PANEL.text_addr(texts::NET_DISABLE_NEW_PLAYERS),
        false,
        2,
    );
    if NET.dplay.get() == 0 {
        return;
    }
    NET.session_desc
        .flags
        .set(NET.session_desc.flags.get() | CLOSED);
    let result =
        { crate::net::dp_objects::set_session_desc(NET.dplay.get(), NET.session_desc.addr(), 0) };
    log_hex(result, true);
}
