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

use crate::errors::Refused;
use crate::field;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::host_session::{OPEN_JOIN, finish, open, session_test};
use crate::net::lobby::{
    DPLAY, JOINED, JOINED_MOST, JOINED_NAME, JOINED_PASSWORD, LOG_HEX_L2_NL, LOG_INT_L1_NL, LOG_L2,
    LOG_L2_NL, POST_CALL, Part, RESTORE_WINDOW, SESSION_DESC, ZERO_MEMORY, a, cmp_flags,
    dispatch_lobby, g, log_text, movsb, p,
};
use crate::net::types::NetSession;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::join_session", rva(0x418C6D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const JOIN: usize = 0x5C2F01; // "NET: Join session"
const COMPLETE: usize = 0x5C2F13; // "NET: Join complete"
pub const POSTED_CODE: u32 = 0x418D77; // the piece posted: `Open` to join
const SESSION_ENTRY: u32 = 0x418553; // net::session_entry
const CREATE_PLAYER: u32 = 0x4190FB; // net::create_player
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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, join_session);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places = [
        LOG_L2,
        LOG_INT_L1_NL,
        SESSION_ENTRY,
        ZERO_MEMORY,
        POST_CALL,
        LOG_HEX_L2_NL,
        RESTORE_WINDOW,
        CREATE_PLAYER,
        JOIN_GROUP,
        LOG_L2_NL,
    ];
    // pusha; pusha; lea eax, aNetJoinSession
    let copy = match crate::net::lobby::copy(
        0x418C6D,
        0x10A,
        &[0x60, 0x60, 0x8D, 0x05, 0x01, 0x2F, 0x5C, 0x00],
        &[
            0x8, 0xF, 0x22, 0x73, 0xA8, 0xC8, 0xCF, 0xD5, 0xE8, 0xEF, 0xFD,
        ],
        &places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let record: Vec<u8> = (0..0x68u8).map(|n| n.wrapping_mul(5) ^ 0x21).collect();
    session_test(
        &mut t,
        copy.addr(),
        Regs {
            eax: 2,
            ..Regs::SENTINEL
        },
        JOIN_GROUP,
        record.as_ptr() as u32,
        &join_session,
    );
    drop(record);
    t.finish()
}
