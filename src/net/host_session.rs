//! Hosts a session ("NET: Host"): keeps its description (name, most players, the game's mark), opens
//! it with DPOPEN_CREATE (directly or, with `posting` 1, on the receiving thread), logs the
//! result and restores the window; then the own player and the group are made. The name goes to
//! `net::dp_objects` as text. Returns true when hosted.

use crate::errors::Refused;
use crate::net::find_sessions::{posted_or_called, zeroed_description};
use crate::net::link::Link;
use crate::net::lobby::{log_bytes, log_hex};
use crate::net::owner::Net;
use crate::net::store::put_bytes;
use crate::race::viewer_screen_store::{NetTexts, texts};

const OPEN_CREATE: u32 = 2; // DPOPEN_CREATE
pub const OPEN_JOIN: u32 = 1; // DPOPEN_JOIN

/// `Open(description, flags)`.
pub fn open(link: &Link, flags: u32, name: &[u8]) -> u32 {
    let d = crate::net::dp_objects::desc(link.session_desc(), name);
    crate::net::dp_objects::open_with_password(
        link.dplay(),
        &d,
        flags,
        &if flags & OPEN_JOIN != 0 {
            crate::net::password::joining()
        } else {
            crate::net::password::get()
        },
    )
}

/// The `Open` as the receiving thread calls it when it is posted.
pub fn posted_open_host(net: &Net, _lines: &NetTexts) -> u32 {
    open(&net.link, OPEN_CREATE, &net.link.host_name().read())
}
pub fn host_session(o: crate::owners::Owners<'static>, lines: &NetTexts) -> Result<bool, Refused> {
    let net = o.net;
    log_bytes(&lines.text(texts::NET_HOST), false, 2);
    log_bytes(&net.setup().machine_name.read(), true, 2);
    if !net.link.has_dplay() {
        return Ok(false);
    }
    net.joined().joined.write(&[0; 16]);
    net.joined()
        .joined_name
        .write(&net.setup().machine_name.read());
    net.joined().joined_most.set(net.setup().most_players.get());
    net.joined()
        .joined_user_values
        .write(&net.setup().session_user_values.read());
    // For `posted_open_host`, which may run on the receiving thread.
    net.link.host_name().write(&net.setup().machine_name.read());
    zeroed_description(&net.link)?;
    let desc = net.link.session_desc();
    desc.flags.set(0x44);
    put_bytes(&desc.application, &crate::net::store::APP_GUID);
    desc.max_players.set(net.setup().most_players.get());
    // The name goes to `open` as text (the description's pointer is left 0).
    put_bytes(&desc.user, &net.setup().session_user_values.read());
    finish(
        o,
        lines,
        posted_open_host,
        crate::net::create_group::create_group,
        texts::NET_HOST_COMPLETE,
    )
}

/// The session opened (posted or called: `posted`), "NET: Open" and its result logged, the window
/// restored; the session marked made, the player made, then `then` (the group made or joined) and
/// the text numbered `done` logged. Shared with `net::join_session`. Whether all of it worked.
pub fn finish(
    o: crate::owners::Owners<'static>,
    lines: &NetTexts,
    posted: crate::net::post_call::PostedCall,
    then: fn(&Link) -> Result<bool, Refused>,
    done: usize,
) -> Result<bool, Refused> {
    let net = o.net;
    let result = posted_or_called(o, lines, posted);
    crate::net::password::note_refused(result == crate::net::dp_session::DPERR_INVALIDPASSWORD);
    log_bytes(&lines.text(texts::NET_OPEN), false, 2);
    log_hex(result, true);
    crate::system::restore_window::restore_window();
    if result != 0 {
        return Ok(false);
    }
    net.link.mark_session_open();
    if !crate::net::create_player::create_player(net, lines)? || !then(&net.link)? {
        return Ok(false);
    }
    log_bytes(&lines.text(done), true, 2);
    Ok(true)
}
