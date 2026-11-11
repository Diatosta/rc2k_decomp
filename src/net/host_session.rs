//! `net::host_session` (0x418B13): "NET: Host" and the machine's name logged; with
//! DirectPlay's object there, the session kept (`joined`: no GUID yet, the name
//! `machine_name`, the most players `most_players`, `session_password` as its user data)
//! and described (the session description: zeroed, size 0x50, flags 0x44, the game's GUID, the most
//! players, the name, the user data) and made (`Open` with DPOPEN_CREATE, or with
//! `posting` 1 posted); "NET: Open" and its result logged, the
//! window restored (`system::restore_window`); then the session is marked made, the player made
//! (`net::create_player`), the group made (`net::create_group`), "NET: Host complete".
//!
//! The session's name goes to `net::dp_objects` as text (`open`): the
//! description's name pointer is left 0.
//!
//! Returns true when hosted.
//! Caller: `net::hosting`.

use crate::errors::Refused;
use crate::net::find_sessions::{posted_or_called, zeroed_description};
use crate::net::lobby::{log, log_bytes, log_hex};
use crate::net::owner::Net;
use crate::net::owner::net;
use crate::net::store::put_bytes;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

const OPEN_CREATE: u32 = 2; // DPOPEN_CREATE
pub const OPEN_JOIN: u32 = 1; // DPOPEN_JOIN

/// `Open(description, flags)`.
pub fn open(flags: u32, name: &[u8]) -> u32 {
    let vars = &net().vars;
    let d = crate::net::dp_objects::desc(&vars.session_desc, name);
    crate::net::dp_objects::open(vars.dplay.get(), &d, flags)
}

/// The code at 0x418C5A as the receiving thread calls it when it is posted.
pub fn posted_open_host() -> u32 {
    open(OPEN_CREATE, &net().setup().machine_name.read())
}
pub fn host_session(net: &Net) -> Result<bool, Refused> {
    log(VIEWER_PANEL.text_ptr(texts::NET_HOST), false, 2);
    log_bytes(&net.setup().machine_name.read(), true, 2);
    if net.vars.dplay.get() == 0 {
        return Ok(false);
    }
    net.joined().joined.write(&[0; 16]);
    net.joined()
        .joined_name
        .write(&net.setup().machine_name.read());
    net.joined().joined_most.set(net.setup().most_players.get());
    net.joined()
        .joined_password
        .write(&net.setup().session_password.read());
    zeroed_description(&net.vars)?;
    let desc = &net.vars.session_desc;
    desc.flags.set(0x44);
    put_bytes(&desc.application, &net.vars.app_guid.read());
    desc.max_players.set(net.setup().most_players.get());
    // The name goes to `open` as text (the description's pointer is left 0).
    put_bytes(&desc.user, &net.setup().session_password.read());
    finish(
        net,
        posted_open_host,
        crate::net::create_group::create_group,
        VIEWER_PANEL.text_ptr(texts::NET_HOST_COMPLETE),
    )
}

/// The session opened (posted or called: `posted`), "NET: Open" and its result logged, the
/// window restored; the session marked made, the player made, then `then` (the group made or
/// joined) and `done` logged. Shared with `net::join_session`. Whether all of it worked.
pub fn finish(
    net: &Net,
    posted: fn() -> u32,
    then: fn() -> Result<bool, Refused>,
    done: crate::pointers::GamePtr,
) -> Result<bool, Refused> {
    let result = posted_or_called(net, posted);
    log(VIEWER_PANEL.text_ptr(texts::NET_OPEN), false, 2);
    log_hex(result, true);
    crate::system::restore_window::restore_window();
    if result != 0 {
        return Ok(false);
    }
    net.vars.session_open.set(1);
    if !crate::net::create_player::create_player()? || !then()? {
        return Ok(false);
    }
    log(done, true, 2);
    Ok(true)
}
