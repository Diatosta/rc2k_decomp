//! `net::host_session` (0x418B13): "NET: Host" and the machine's name logged; with
//! DirectPlay's object there, the session kept (`unk_52AD00`: no GUID yet, the name
//! `g_machineName`, the most players `dword_52ABD0`, `unk_52ABD4` as its user data) and
//! described (`dword_5AB2AC`: zeroed, size 0x50, flags 0x44, the game's GUID, the most
//! players, the name, the user data) and made (`Open` with DPOPEN_CREATE, or with
//! `g_netPosting` 1 posted: the code at 0x418C5A); "NET: Open" and its result logged, the
//! window restored (`system::restore_window`); then `dword_52ACFC` 1, the player made
//! (`net::create_player`), the group made (`net::create_group`), "NET: Host complete".
//!
//! Since 2026-10-29 the session's name goes to `net::dp_objects` as text (`open`): the
//! description's name pointer is left 0.
//!
//! In:  nothing
//! Out: true when hosted (the original: carry clear, the flags of `test eax, eax`; every
//!      register kept), false for the carry
//! Caller: `net::hosting`.

use crate::errors::Refused;
use crate::net::find_sessions::{posted_or_called, zeroed_description};
use crate::net::lobby::{log, log_bytes, log_hex};
use crate::net::store::{NET, put_bytes};
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

const OPEN_CREATE: u32 = 2; // DPOPEN_CREATE
pub const OPEN_JOIN: u32 = 1; // DPOPEN_JOIN

/// `Open(dword_5AB2AC, flags)`.
pub fn open(flags: u32, name: &[u8]) -> u32 {
    let d = crate::net::dp_objects::desc(&NET.session_desc, name);
    crate::net::dp_objects::open(NET.dplay.get(), &d, flags)
}

/// The code at 0x418C5A as the receiving thread calls it when it is posted.
pub fn posted_open_host() -> u32 {
    open(OPEN_CREATE, &NET.machine_name.read())
}

/// The Rust version.
pub fn host_session() -> Result<bool, Refused> {
    log(VIEWER_PANEL.text_addr(texts::NET_HOST), false, 2);
    log_bytes(&NET.machine_name.read(), true, 2);
    if NET.dplay.get() == 0 {
        return Ok(false);
    }
    NET.joined.write(&[0; 16]);
    NET.joined_name.write(&NET.machine_name.read());
    NET.joined_most.set(NET.most_players.get());
    NET.joined_password.write(&NET.session_password.read());
    zeroed_description()?;
    let desc = &NET.session_desc;
    desc.flags.set(0x44);
    put_bytes(&desc.application, &NET.app_guid.read());
    desc.max_players.set(NET.most_players.get());
    // The name goes to `open` as text (the description's pointer is left 0).
    put_bytes(&desc.user, &NET.session_password.read());
    finish(
        posted_open_host,
        crate::net::create_group::create_group,
        VIEWER_PANEL.text_addr(texts::NET_HOST_COMPLETE),
    )
}

/// The session opened (posted or called: `posted`), "NET: Open" and its result logged, the
/// window restored; `dword_52ACFC` 1, the player made, then `then` (the group made or
/// joined) and `done` logged. Shared with `net::join_session`. Whether all of it worked.
pub fn finish(
    posted: fn() -> u32,
    then: fn() -> Result<bool, Refused>,
    done: u32,
) -> Result<bool, Refused> {
    let result = posted_or_called(posted);
    log(VIEWER_PANEL.text_addr(texts::NET_OPEN), false, 2);
    log_hex(result, true);
    crate::system::restore_window::restore_window();
    if result != 0 {
        return Ok(false);
    }
    NET.session_open.set(1);
    if !crate::net::create_player::create_player()? || !then()? {
        return Ok(false);
    }
    log(done, true, 2);
    Ok(true)
}
