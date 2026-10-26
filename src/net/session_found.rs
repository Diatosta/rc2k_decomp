//! `net::session_found` (0x418A62): DirectPlay's `EnumSessions` callback: "NET: Found", the
//! flags, the session's name logged; on a time-out (flag 1) or no session the enumeration
//! stops; a session whose `dwUser2` bit 0 is clear is passed over; else, if fewer than
//! 0x100 are listed, a record added (`net::session_entry`): the name
//! (`strings::copy_printable`, 0x40 room), the instance GUID (+0x40), the most players
//! (+0x50), the players (+0x54), `dwUser1` .. `dwUser4` (+0x58).
//!
//! In:  (stdcall) the session's description (a DPSESSIONDESC2), the time-out's address,
//!      the flags, the context
//! Out: eax = 1 to go on, 0 to stop; ebx, esi, edi, ebp kept; `ret 10h`
//! Caller: DirectPlay, from `net::find_sessions` (`callback`, a `stdcall` Rust function).

use crate::errors::Refused;
use crate::field;
use crate::net::lobby::{copy_printable, log, log_hex, movsb_from_rust, session};
use crate::net::store::NET;
use crate::net::types::{DpSessionDesc, NetSession};
use crate::pointers::GamePtr;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};
use rc2k_core::net::lists::{self, Found};

/// A dword of the callback's argument (memory Rust owns: DirectPlay's session record).
fn get(at: u32) -> u32 {
    crate::pointers::read_or_end(at, "DirectPlay's session record")
}

/// The Rust version: eax.
pub fn session_found(args: &[u32]) -> Result<u32, Refused> {
    let (desc, flags) = (args[0], args[2]);
    log(VIEWER_PANEL.text_addr(texts::NET_FOUND), false, 2);
    log_hex(flags, false);
    if desc != 0 {
        log(get(desc + 0x30), false, 2);
    }
    log(c"".as_ptr() as u32, true, 2);
    // Converted: `rc2k_core::net::lists::session_found`.
    let count =
        match lists::session_found(flags, desc != 0, || get(desc + 0x44), || NET.sessions.get()) {
            Found::Stop => return Ok(0),
            Found::Pass => return Ok(1),
            Found::List(n) => n,
        };
    NET.sessions.set(count + 1);
    let record = session(count)?;
    copy_printable(get(desc + 0x30), record, 0x40)?;
    movsb_from_rust(
        record + field!(NetSession, instance),
        desc + field!(DpSessionDesc, instance),
        0x10,
    )?;
    GamePtr::new(record + field!(NetSession, max_players)).write(
        get(desc + field!(DpSessionDesc, max_players)),
        "a session's record",
    )?;
    GamePtr::new(record + field!(NetSession, players)).write(
        get(desc + field!(DpSessionDesc, current_players)),
        "a session's record",
    )?;
    movsb_from_rust(
        record + field!(NetSession, user),
        desc + field!(DpSessionDesc, user),
        0x10,
    )?;
    Ok(1)
}

/// The callback DirectPlay is given: the Rust version.
pub unsafe extern "system" fn callback(a: u32, b: u32, c: u32, d: u32) -> u32 {
    crate::net::lobby::callback_entry("net::session_found", 0x418A62, &[a, b, c, d], session_found)
}
