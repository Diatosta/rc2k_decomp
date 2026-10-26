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
use crate::net::lobby::{log, log_int_nl, movsb, session, zero};
use crate::net::store::NET;
use crate::net::types::NetSession;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

/// The code at 0x418D77 as the receiving thread calls it when it is posted.
pub extern "C" fn posted_open_join() -> u32 {
    unsafe { open(OPEN_JOIN) }
}

fn join_group() -> Result<bool, Refused> {
    Ok(crate::net::join_group::join_group())
}

/// The Rust version.
pub fn join_session(number: u32) -> Result<bool, Refused> {
    log(VIEWER_PANEL.text_addr(texts::NET_JOIN_SESSION), false, 2);
    log_int_nl(number, 1);
    if NET.dplay.get() == 0 {
        return Ok(false);
    }
    let record = session(number)?;
    movsb(
        NET.joined.addr(),
        record + field!(NetSession, instance),
        0x10,
    )?;
    movsb(NET.joined_name.addr(), record, 0x40)?;
    NET.joined_most
        .set(crate::pointers::GamePtr::new(record + 0x50).read::<u32>("a session's record")?);
    movsb(
        NET.joined_password.addr(),
        record + field!(NetSession, user),
        0x10,
    )?;
    zero(NET.session_desc.addr(), 0x50)?;
    NET.session_desc.size.set(0x50);
    movsb(
        crate::pointers::game_address(NET.session_desc.instance.as_ptr()),
        NET.joined.addr(),
        0x10,
    )?;
    finish(
        posted_open_join,
        join_group,
        VIEWER_PANEL.text_addr(texts::NET_JOIN_COMPLETE),
    )
}
