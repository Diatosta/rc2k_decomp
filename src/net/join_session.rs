//! `net::join_session` (0x418C6D): "NET: Join session" and the session's number logged; with
//! DirectPlay's object there, session eax's record (`net::session_entry`) kept
//! (`unk_52AD00`: its GUID, name, most players, user data), a session description
//! (`dword_5AB2AC`: zeroed, size 0x50, the instance GUID) made and the session joined
//! (`Open` with DPOPEN_JOIN, or with `g_netPosting` 1 posted: the code at 0x418D77);
//! "NET: Open" and its result logged, the window restored; then `dword_52ACFC` 1, the player
//! made (`net::create_player`), the group joined (`net::join_group`), "NET: Join complete".
//!
//! In:  `session` (eax: the session's number)
//! Out: true when joined (the original: carry clear, the flags of `test eax, eax`; every
//!      register kept), false for the carry
//! Caller: `net::joining`.

use crate::errors::Refused;
use crate::field;
use crate::net::host_session::{OPEN_JOIN, finish, open};
use crate::net::lobby::{
    DPLAY, JOINED, JOINED_MOST, JOINED_NAME, JOINED_PASSWORD, SESSION_DESC, a, g, log, log_int_nl,
    movsb, p, session, zero,
};
use crate::net::types::NetSession;

const JOIN: usize = 0x5C2F01; // "NET: Join session"
const COMPLETE: usize = 0x5C2F13; // "NET: Join complete"

/// The code at 0x418D77 as the receiving thread calls it when it is posted.
pub extern "C" fn posted_open_join() -> u32 {
    unsafe { open(OPEN_JOIN) }
}

fn join_group() -> Result<bool, Refused> {
    Ok(crate::net::join_group::join_group())
}

/// The Rust version.
pub fn join_session(number: u32) -> Result<bool, Refused> {
    log(a(JOIN), false, 2);
    log_int_nl(number, 1);
    if g(DPLAY) == 0 {
        return Ok(false);
    }
    let record = session(number)?;
    movsb(a(JOINED), record + field!(NetSession, instance), 0x10);
    movsb(a(JOINED_NAME), record, 0x40);
    p(JOINED_MOST, unsafe {
        ((record + 0x50) as *const u32).read_unaligned()
    });
    movsb(a(JOINED_PASSWORD), record + field!(NetSession, user), 0x10);
    zero(a(SESSION_DESC), 0x50)?;
    p(SESSION_DESC, 0x50);
    movsb(a(SESSION_DESC) + 8, a(JOINED), 0x10);
    finish(posted_open_join, join_group, COMPLETE)
}
