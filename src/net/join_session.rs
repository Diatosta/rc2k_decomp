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
use crate::net::host_session::{OPEN_JOIN, finish, open};
use crate::net::lobby::{log, log_int_nl};
use crate::net::owner::net;
use crate::net::store::put_bytes;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

/// The code at 0x418D77 as the receiving thread calls it when it is posted.
pub fn posted_open_join() -> u32 {
    open(OPEN_JOIN, b"")
}

fn join_group() -> Result<bool, Refused> {
    Ok(crate::net::join_group::join_group())
}

/// The Rust version.
pub fn join_session(number: u32) -> Result<bool, Refused> {
    let net = net();
    log(VIEWER_PANEL.text_ptr(texts::NET_JOIN_SESSION), false, 2);
    log_int_nl(number, 1);
    if net.vars.dplay.get() == 0 {
        return Ok(false);
    }
    let s = crate::net::tables::with_session(number, |s| *s)?;
    net.vars
        .joined
        .write_at(0, &s.instance, "the joined session")?;
    net.vars
        .joined_name
        .write_at(0, &s.name, "the joined session")?;
    net.vars.joined_most.set(s.max_players);
    net.vars
        .joined_password
        .write_at(0, &s.user, "the joined session")?;
    net.vars.session_desc.clear();
    net.vars.session_desc.size.set(0x50);
    put_bytes(&net.vars.session_desc.instance, &net.vars.joined.read());
    finish(
        posted_open_join,
        join_group,
        VIEWER_PANEL.text_ptr(texts::NET_JOIN_COMPLETE),
    )
}
