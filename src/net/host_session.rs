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
use crate::field;
use crate::net::dplay::{OPEN, call};
use crate::net::find_sessions::{posted_or_called, zeroed_description};
use crate::net::lobby::{
    APP_GUID, DPLAY, JOINED, JOINED_MOST, JOINED_NAME, JOINED_PASSWORD, MACHINE_NAME, MOST_PLAYERS,
    SESSION_DESC, SESSION_OPEN, SESSION_PASSWORD, a, g, log, log_hex, movsb, p, zero,
};
use crate::net::types::DpSessionDesc;

const HOST: usize = 0x5C2EDA; // "NET: Host"
pub const OPENED: usize = 0x5C2EE4; // "NET: Open"
const COMPLETE: usize = 0x5C2EEE; // "NET: Host complete"
const OPEN_CREATE: u32 = 2; // DPOPEN_CREATE
pub const OPEN_JOIN: u32 = 1; // DPOPEN_JOIN

/// `Open(dword_5AB2AC, flags)`.
pub unsafe fn open(flags: u32) -> u32 {
    unsafe { call(DPLAY, OPEN, &[a(SESSION_DESC), flags]) }
}

/// The code at 0x418C5A as the receiving thread calls it when it is posted.
pub extern "C" fn posted_open_host() -> u32 {
    unsafe { open(OPEN_CREATE) }
}

/// The Rust version.
pub fn host_session() -> Result<bool, Refused> {
    log(a(HOST), false, 2);
    log(a(MACHINE_NAME), true, 2);
    if g(DPLAY) == 0 {
        return Ok(false);
    }
    zero(a(JOINED), 0x10)?;
    movsb(a(JOINED_NAME), a(MACHINE_NAME), 0x40);
    p(JOINED_MOST, g(MOST_PLAYERS));
    movsb(a(JOINED_PASSWORD), a(SESSION_PASSWORD), 0x10);
    zeroed_description()?;
    p(SESSION_DESC + 4, 0x44);
    movsb(a(SESSION_DESC) + 0x18, a(APP_GUID), 0x10);
    p(SESSION_DESC + 0x28, g(MOST_PLAYERS));
    p(SESSION_DESC + 0x30, a(MACHINE_NAME));
    movsb(
        a(SESSION_DESC) + field!(DpSessionDesc, user),
        a(SESSION_PASSWORD),
        0x10,
    );
    finish(
        posted_open_host,
        crate::net::create_group::create_group,
        COMPLETE,
    )
}

/// The session opened (posted or called: `posted`), "NET: Open" and its result logged, the
/// window restored; `dword_52ACFC` 1, the player made, then `then` (the group made or
/// joined) and `done` logged. Shared with `net::join_session`. Whether all of it worked.
pub fn finish(
    posted: extern "C" fn() -> u32,
    then: fn() -> Result<bool, Refused>,
    done: usize,
) -> Result<bool, Refused> {
    let result = posted_or_called(posted);
    log(a(OPENED), false, 2);
    log_hex(result, true);
    crate::system::restore_window::restore_window();
    if result != 0 {
        return Ok(false);
    }
    p(SESSION_OPEN, 1);
    if !crate::net::create_player::create_player()? || !then()? {
        return Ok(false);
    }
    log(a(done), true, 2);
    Ok(true)
}
