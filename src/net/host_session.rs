//! `net::host_session` (0x418B13): "NET: Host" and the machine's name logged; with
//! DirectPlay's object there, the session kept (`unk_52AD00`: no GUID yet, the name
//! `g_machineName`, the most players `dword_52ABD0`, `unk_52ABD4` as its user data) and
//! described (`dword_5AB2AC`: zeroed, size 0x50, flags 0x44, the game's GUID, the most
//! players, the name, the user data) and made (`Open` with DPOPEN_CREATE, or with
//! `g_netPosting` 1 posted: the code at 0x418C5A); "NET: Open" and its result logged, the
//! window restored (`system::restore_window`); then `dword_52ACFC` 1, the player made
//! (`net::create_player`), the group made (`net::create_group`), "NET: Host complete".
//!
//! In:  nothing
//! Out: true when hosted (the original: carry clear, the flags of `test eax, eax`; every
//!      register kept), false for the carry
//! Caller: `net::hosting`.

use crate::errors::Refused;
use crate::net::dplay::{OPEN, call};
use crate::net::find_sessions::{posted_or_called, zeroed_description};
use crate::net::lobby::{log, log_hex, movsb, zero};
use crate::net::store::NET;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

const OPEN_CREATE: u32 = 2; // DPOPEN_CREATE
pub const OPEN_JOIN: u32 = 1; // DPOPEN_JOIN

/// `Open(dword_5AB2AC, flags)`.
pub unsafe fn open(flags: u32) -> u32 {
    unsafe { call(&NET.dplay, OPEN, &[NET.session_desc.addr() as u32, flags]) }
}

/// The code at 0x418C5A as the receiving thread calls it when it is posted.
pub extern "C" fn posted_open_host() -> u32 {
    unsafe { open(OPEN_CREATE) }
}

/// The Rust version.
pub fn host_session() -> Result<bool, Refused> {
    log(VIEWER_PANEL.text_addr(texts::NET_HOST), false, 2);
    log(NET.machine_name.addr() as u32, true, 2);
    if NET.dplay.get() == 0 {
        return Ok(false);
    }
    zero(NET.joined.addr() as u32, 0x10)?;
    movsb(
        NET.joined_name.addr() as u32,
        NET.machine_name.addr() as u32,
        0x40,
    )?;
    NET.joined_most.set(NET.most_players.get());
    movsb(
        NET.joined_password.addr() as u32,
        NET.session_password.addr() as u32,
        0x10,
    )?;
    zeroed_description()?;
    NET.session_desc.flags.set(0x44);
    movsb(
        NET.session_desc.application.as_ptr() as u32,
        NET.app_guid.addr() as u32,
        0x10,
    )?;
    NET.session_desc.max_players.set(NET.most_players.get());
    NET.session_desc.name.set(NET.machine_name.addr() as u32);
    movsb(
        NET.session_desc.user.as_ptr() as u32,
        NET.session_password.addr() as u32,
        0x10,
    )?;
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
    posted: extern "C" fn() -> u32,
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
