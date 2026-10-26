//! `net::join_session` (0x418C6D): "NET: Join session" and the session's number logged; with
//! DirectPlay's object there, session eax's record (`net::session_entry`) kept
//! (`unk_52AD00`: its GUID, name, most players, user data), a session description
//! (`dword_5AB2AC`: zeroed, size 0x50, the instance GUID) made and the session joined
//! (`Open` with DPOPEN_JOIN, or with `g_netPosting` 1 posted: the code at 0x418D77);
//! "NET: Open" and its result logged, the window restored; then `dword_52ACFC` 1, the player
//! made (`net::create_player`), the group joined (`net::join_group`), "NET: Join complete".
//!
//! In:  eax = the session's number
//! Out: every register kept (`pusha` .. `popa`); carry set if something failed, else the
//!      flags of `test eax, eax`
//! Caller: `net::joining`. Its calls through `part`; `Open` from Rust (`net::dplay`; posted,
//! `posted_open_join`). Shadow mode runs the original alone. Self-test: as
//! `net::host_session`'s, the session's record a test one.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::field;
use crate::framework::Regs;
use crate::net::host_session::{OPEN_JOIN, finish, open};
use crate::net::lobby::{
    DPLAY, JOINED, JOINED_MOST, JOINED_NAME, JOINED_PASSWORD, LOG_INT_L1_NL, LOG_L2, Part,
    SESSION_DESC, ZERO_MEMORY, a, cmp_flags, g, lobby_entry, log_text, movsb, p,
};
use crate::net::types::NetSession;

const JOIN: usize = 0x5C2F01; // "NET: Join session"
const COMPLETE: usize = 0x5C2F13; // "NET: Join complete"
pub const POSTED_CODE: u32 = 0x418D77; // the piece posted: `Open` to join
const SESSION_ENTRY: u32 = 0x418553; // net::session_entry
// net::create_player
const JOIN_GROUP: u32 = 0x419354; // net::join_group

/// The code at 0x418D77 as the receiving thread calls it when it is posted.
pub extern "C" fn posted_open_join() -> u32 {
    unsafe { open(OPEN_JOIN) }
}

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn join_session(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs| Regs {
        eflags: r.eflags,
        ..*entry
    };
    log_text(LOG_L2, a(JOIN), &r, part)?;
    let mut l = r;
    part(LOG_INT_L1_NL, &mut l)?;
    cmp_flags(&mut r, g(DPLAY), 0);
    if g(DPLAY) == 0 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    part(SESSION_ENTRY, &mut r)?;
    r.esi = r.eax;
    let record = r.esi;
    movsb(a(JOINED), record + field!(NetSession, instance), 0x10);
    movsb(a(JOINED_NAME), record, 0x40);
    r.eax = unsafe { ((record + 0x50) as *const u32).read_unaligned() }; // mov eax, [esi+50h]
    p(JOINED_MOST, r.eax);
    movsb(a(JOINED_PASSWORD), record + field!(NetSession, user), 0x10);
    let mut c = Regs {
        ebx: a(SESSION_DESC),
        ecx: 0x50,
        ..r
    };
    part(ZERO_MEMORY, &mut c)?;
    p(SESSION_DESC, 0x50);
    movsb(a(SESSION_DESC) + 8, a(JOINED), 0x10);
    finish(&mut r, OPEN_JOIN, POSTED_CODE, JOIN_GROUP, COMPLETE, part)?;
    Ok(left(&r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, join_session)
}
