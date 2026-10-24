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
use crate::net::lobby::{SESSIONS, a, copy_printable, g, log, log_hex, movsb, p, session};
use crate::net::types::{DpSessionDesc, NetSession};
use rc2k_core::net::lists::{self, Found};

const FOUND: usize = 0x5C2E75; // "NET: Found"
const NEWLINE: u32 = 0x4E00B7; // the text `logfile::log_newline_l2` writes

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The Rust version: eax.
pub fn session_found(args: &[u32]) -> Result<u32, Refused> {
    let (desc, flags) = (args[0], args[2]);
    log(a(FOUND), false, 2);
    log_hex(flags, false);
    if desc != 0 {
        log(get(desc + 0x30), false, 2);
    }
    log(crate::net::lobby::game(NEWLINE), true, 2);
    // Converted: `rc2k_core::net::lists::session_found`.
    let count = match lists::session_found(flags, desc != 0, || get(desc + 0x44), || g(SESSIONS)) {
        Found::Stop => return Ok(0),
        Found::Pass => return Ok(1),
        Found::List(n) => n,
    };
    p(SESSIONS, count + 1);
    let record = session(count)?;
    copy_printable(get(desc + 0x30), record, 0x40)?;
    movsb(
        record + field!(NetSession, instance),
        desc + field!(DpSessionDesc, instance),
        0x10,
    );
    unsafe {
        ((record + field!(NetSession, max_players)) as *mut u32)
            .write_unaligned(get(desc + field!(DpSessionDesc, max_players)));
        ((record + field!(NetSession, players)) as *mut u32)
            .write_unaligned(get(desc + field!(DpSessionDesc, current_players)));
    }
    movsb(
        record + field!(NetSession, user),
        desc + field!(DpSessionDesc, user),
        0x10,
    );
    Ok(1)
}

/// The callback DirectPlay is given: the Rust version.
pub unsafe extern "system" fn callback(a: u32, b: u32, c: u32, d: u32) -> u32 {
    crate::net::lobby::callback_entry("net::session_found", 0x418A62, &[a, b, c, d], session_found)
}
