//! `net::disable_new_players` (0x418D8A): on the host (state bit 0x10 of `byte_52AD64`), "NET:
//! Disable new players", and with DirectPlay's object there the session's description
//! (`dword_5AB2AC`) given DPSESSION_NEWPLAYERSDISABLED and DPSESSION_JOINDISABLED (0x21) and
//! set (`SetSessionDesc`, flags 0); its result logged.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`; the carry it sets when not the
//!      host, with no object or on a failure is read by no caller: `lq -s 0x418D8A`)
//! Caller: `net::close_if_host`.

use crate::net::dplay::{SET_SESSION_DESC, call};
use crate::net::lobby::{DPLAY, OWN, SESSION_DESC, a, g, gb, log, log_hex, p};

const TEXT: usize = 0x5C2F26; // "NET: Disable new players"
const CLOSED: u32 = 0x21; // DPSESSION_NEWPLAYERSDISABLED | DPSESSION_JOINDISABLED

/// The Rust version.
pub fn disable_new_players() {
    if gb(OWN) & 0x10 == 0 {
        return;
    }
    log(a(TEXT), false, 2);
    if g(DPLAY) == 0 {
        return;
    }
    p(SESSION_DESC + 4, g(SESSION_DESC + 4) | CLOSED);
    let result = unsafe { call(DPLAY, SET_SESSION_DESC, &[a(SESSION_DESC), 0]) };
    log_hex(result, true);
}
