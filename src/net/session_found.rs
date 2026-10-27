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
use crate::net::lobby::{log, log_hex};
use crate::net::store::NET;
use crate::net::tables::{copy_printable_into, with_session};
use crate::net::types::DpSessionDesc;
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
    log(crate::pointers::game_address(c"".as_ptr()), true, 2);
    // Converted: `rc2k_core::net::lists::session_found`.
    let count =
        match lists::session_found(flags, desc != 0, || get(desc + 0x44), || NET.sessions.get()) {
            Found::Stop => return Ok(0),
            Found::Pass => return Ok(1),
            Found::List(n) => n,
        };
    NET.sessions.set(count + 1);
    let mut name = with_session(count, |s| s.name)?;
    copy_printable_into(get(desc + 0x30), &mut name, 0x40)?;
    let bytes = |at: u32| GamePtr::new(desc + at).read::<[u8; 16]>("DirectPlay's session record");
    let (instance, user) = (
        bytes(field!(DpSessionDesc, instance))?,
        bytes(field!(DpSessionDesc, user))?,
    );
    let (most, players) = (
        get(desc + field!(DpSessionDesc, max_players)),
        get(desc + field!(DpSessionDesc, current_players)),
    );
    with_session(count, |s| {
        s.name = name;
        s.instance = instance;
        s.max_players = most;
        s.players = players;
        s.user = user;
    })?;
    Ok(1)
}

/// The callback DirectPlay is given (`net::dp_objects`): the Rust version.
pub fn callback(a: u32, b: u32, c: u32, d: u32) -> u32 {
    crate::net::lobby::callback_entry("net::session_found", 0x418A62, &[a, b, c, d], session_found)
}
