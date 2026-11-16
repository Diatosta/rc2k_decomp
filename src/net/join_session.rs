//! Joins a session ("NET: Join session"): keeps its record, opens it with DPOPEN_JOIN
//! (directly or, with `posting` 1, on the receiving thread), logs the result and restores the
//! window; then the own player is made and the group joined. Returns true when joined.

use crate::errors::Refused;
use crate::net::host_session::{OPEN_JOIN, finish, open};
use crate::net::lobby::{log_bytes, log_int_nl};
use crate::net::owner::Net;
use crate::net::store::put_bytes;
use crate::race::viewer_screen_store::{NetTexts, texts};

/// The `Open` as the receiving thread calls it when it is posted.
pub fn posted_open_join(net: &Net, _lines: &NetTexts) -> u32 {
    open(&net.link, OPEN_JOIN, b"")
}

fn join_group(link: &crate::net::link::Link) -> Result<bool, Refused> {
    Ok(crate::net::join_group::join_group(link))
}
pub fn join_session(
    o: crate::owners::Owners<'static>,
    lines: &NetTexts,
    number: u32,
) -> Result<bool, Refused> {
    let net = o.net;
    log_bytes(&lines.text(texts::NET_JOIN_SESSION), false, 2);
    log_int_nl(number, 1);
    if !net.link.has_dplay() {
        return Ok(false);
    }
    let s = crate::net::tables::with_session(&net.link, number, |s| *s)?;
    net.joined()
        .joined
        .write_at(0, &s.instance, "the joined session")?;
    net.joined()
        .joined_name
        .write_at(0, &s.name, "the joined session")?;
    net.joined().joined_most.set(s.max_players);
    net.joined()
        .joined_user_values
        .write_at(0, &s.user, "the joined session")?;
    net.link.session_desc().clear();
    net.link.session_desc().size.set(0x50);
    put_bytes(
        &net.link.session_desc().instance,
        &net.joined().joined.read(),
    );
    finish(
        o,
        lines,
        posted_open_join,
        join_group,
        texts::NET_JOIN_COMPLETE,
    )
}
