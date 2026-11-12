//! `net::session_found` (0x418A62): DirectPlay's `EnumSessions` callback: "NET: Found", the
//! flags, the session's name logged; on a time-out (flag 1) or no session the enumeration
//! stops; a session whose `dwUser2` bit 0 is clear is passed over; else, if fewer than
//! 0x100 are listed, a record added (`net::session_entry`): the name
//! (`strings::copy_printable`, 0x40 room), the instance GUID (+0x40), the most players
//! (+0x50), the players (+0x54), `dwUser1` .. `dwUser4` (+0x58).
//!
//! Takes (stdcall) the session's description (a DPSESSIONDESC2), the time-out's address, the
//! flags and the context. Returns 1 to go on, 0 to stop.
//! Caller: DirectPlay, from `net::find_sessions` (`callback`, a `stdcall` Rust function).

use crate::errors::Refused;
use crate::net::dp_session::Desc;
use crate::net::lobby::{log_bytes, log_hex};
use crate::net::owner::net;
use crate::net::tables::{copy_printable_into, with_session};
use crate::race::viewer_screen_store::{ViewerPanelBlock, texts};
use rc2k_core::net::lists::{self, Found};

/// The session (none at the end) and the flags.
pub fn session_found(
    panel: &ViewerPanelBlock,
    desc: Option<&Desc>,
    flags: u32,
) -> Result<u32, Refused> {
    let vars = &net().vars;
    log_bytes(&panel.text(texts::NET_FOUND), false, 2);
    log_hex(flags, false);
    if let Some(d) = desc {
        log_bytes(d.name.as_bytes(), false, 2);
    }
    log_bytes(b"", true, 2);
    let user2 = || desc.map_or(0, |d| d.user[1]);
    let count = match lists::session_found(flags, desc.is_some(), user2, || vars.sessions.get()) {
        Found::Stop => return Ok(0),
        Found::Pass => return Ok(1),
        Found::List(n) => n,
    };
    let Some(d) = desc else {
        return Ok(0);
    };
    vars.sessions.set(count + 1);
    let mut name = with_session(vars, count, |s| s.name)?;
    copy_printable_into(d.name.as_bytes(), &mut name, 0x40)?;
    let mut user = [0u8; 16];
    for (k, v) in d.user.iter().enumerate() {
        user[4 * k..4 * k + 4].copy_from_slice(&v.to_le_bytes());
    }
    with_session(vars, count, |s| {
        s.name = name;
        s.instance = d.instance;
        s.max_players = d.max_players;
        s.players = d.current_players;
        s.user = user;
    })?;
    Ok(1)
}

pub fn callback(desc: Option<&Desc>, flags: u32, _context: u32) -> u32 {
    crate::net::lobby::callback_entry("net::session_found", 0x418A62, || {
        session_found(crate::race::owner::race().viewer_panel(), desc, flags)
    })
}
