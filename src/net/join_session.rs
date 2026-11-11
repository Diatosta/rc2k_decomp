//! `net::join_session` (0x418C6D): "NET: Join session" and the session's number logged; with
//! DirectPlay's object there, the session's record (`net::session_entry`) kept
//! (`joined`: its GUID, name, most players, user data), a session description
//! (zeroed, size 0x50, the instance GUID) made and the session joined
//! (`Open` with DPOPEN_JOIN, or with `posting` 1 posted);
//! "NET: Open" and its result logged, the window restored; then the session marked made, the player
//! made (`net::create_player`), the group joined (`net::join_group`), "NET: Join complete".
//!
//! Takes `session`. Returns true when joined.
//! Caller: `net::joining`.

use crate::errors::Refused;
use crate::net::host_session::{OPEN_JOIN, finish, open};
use crate::net::lobby::{log_bytes, log_int_nl};
use crate::net::owner::Net;
use crate::net::store::put_bytes;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

/// The code at 0x418D77 as the receiving thread calls it when it is posted.
pub fn posted_open_join() -> u32 {
    open(OPEN_JOIN, b"")
}

fn join_group() -> Result<bool, Refused> {
    Ok(crate::net::join_group::join_group())
}
pub fn join_session(net: &Net, number: u32) -> Result<bool, Refused> {
    log_bytes(&VIEWER_PANEL.text(texts::NET_JOIN_SESSION), false, 2);
    log_int_nl(number, 1);
    if net.vars.dplay.get() == 0 {
        return Ok(false);
    }
    let s = crate::net::tables::with_session(number, |s| *s)?;
    net.joined()
        .joined
        .write_at(0, &s.instance, "the joined session")?;
    net.joined()
        .joined_name
        .write_at(0, &s.name, "the joined session")?;
    net.joined().joined_most.set(s.max_players);
    net.joined()
        .joined_password
        .write_at(0, &s.user, "the joined session")?;
    net.vars.session_desc.clear();
    net.vars.session_desc.size.set(0x50);
    put_bytes(&net.vars.session_desc.instance, &net.joined().joined.read());
    finish(net, posted_open_join, join_group, texts::NET_JOIN_COMPLETE)
}
