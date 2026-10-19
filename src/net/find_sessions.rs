//! `net::find_sessions` (0x418999): "NET: Find sessions"; with DirectPlay's object there, the
//! sessions listed afresh (`dword_52ACF4` 0): a session description (`dword_5AB2AC`, 0x50
//! bytes, zeroed, its size 0x50, the game's GUID `dword_5AB300` at +0x18) and
//! `EnumSessions` (no timeout, `net::session_found`, flags 0x91) called, or with
//! `g_netPosting` 1 posted to the receiving thread (`net::post_call`, the code at 0x418A43);
//! then the window restored (`system::restore_window`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if there is no object, the user
//!      cancelled (DPERR_USERCANCEL) or it failed; else (done, or DPERR_CONNECTING) the
//!      flags of `test eax, eax`
//! Caller: `net::find_sessions_regs`. Its calls through `part`; `EnumSessions` from Rust
//! (`net::dplay`; posted, `posted_enum_sessions`). Shadow mode runs the original alone.
//! Self-test: a copy whose calls go to stand-ins and whose DirectPlay is a test object,
//! against the Rust version: no object, posting, each result.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::dplay::{ENUM_SESSIONS, call};
use crate::net::lobby::{
    APP_GUID, DPLAY, DPLAY_METHOD, FLAGS, LOG_L2_NL, POST_CALL, POSTING, Part, RESTORE_WINDOW,
    SESSION_DESC, SESSIONS, Saved, TestObject, ZERO_MEMORY, a, answer, cmp_flags, com_answer,
    dispatch_lobby, dword, fill, g, game, log_text, movsb, p, region, run_both, test_eax,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::find_sessions", rva(0x418999), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TEXT: usize = 0x5C2EC7; // "NET: Find sessions"
pub const POSTED_CODE: u32 = 0x418A43; // the piece posted: `EnumSessions`
const SESSION_FOUND: u32 = 0x418A62; // net::session_found
const ENUM_FLAGS: u32 = 0x91;
pub const USER_CANCEL: u32 = 0x8877_0118; // DPERR_USERCANCEL
const CONNECTING: u32 = 0x8877_015E; // DPERR_CONNECTING

/// The session description made ready: zeroed, its size, the game's GUID.
pub fn zeroed_description(r: &Regs, part: Part) -> Result<(), Refused> {
    let mut c = Regs {
        ebx: a(SESSION_DESC),
        ecx: 0x50,
        ..*r
    };
    part(ZERO_MEMORY, &mut c)?;
    p(SESSION_DESC, 0x50);
    Ok(())
}

/// `EnumSessions` as the code at 0x418A43 calls it.
pub unsafe fn enum_sessions() -> u32 {
    unsafe {
        call(
            DPLAY,
            ENUM_SESSIONS,
            &[a(SESSION_DESC), 0, game(SESSION_FOUND), 0, ENUM_FLAGS],
        )
    }
}

/// The code at 0x418A43 as the receiving thread calls it when it is posted.
pub extern "C" fn posted_enum_sessions() -> u32 {
    unsafe { enum_sessions() }
}

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn find_sessions(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs| Regs {
        eflags: r.eflags,
        ..*entry
    };
    log_text(LOG_L2_NL, a(TEXT), &r, part)?;
    cmp_flags(&mut r, g(DPLAY), 0);
    if g(DPLAY) == 0 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    p(SESSIONS, 0);
    zeroed_description(&r, part)?;
    movsb(a(SESSION_DESC) + 0x18, a(APP_GUID), 0x10);
    if g(POSTING) == 1 {
        r.eax = game(POSTED_CODE);
        part(POST_CALL, &mut r)?;
    } else {
        r.eax = unsafe { enum_sessions() };
    }
    part(RESTORE_WINDOW, &mut r)?;
    if r.eax == USER_CANCEL {
        let v = r.eax;
        cmp_flags(&mut r, v, USER_CANCEL);
        r.set_carry(true);
        return Ok(left(&r));
    }
    if r.eax != CONNECTING && r.eax != 0 {
        test_eax(&mut r);
        r.set_carry(true);
        return Ok(left(&r));
    }
    test_eax(&mut r);
    Ok(left(&r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, find_sessions);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; pusha; lea eax, aNetFindSession
    let copy = match crate::net::lobby::copy(
        0x418999,
        0xAA,
        &[0x60, 0x60, 0x8D, 0x05, 0xC7, 0x2E, 0x5C, 0x00],
        &[0x8, 0x32, 0x67, 0x8C],
        &[LOG_L2_NL, ZERO_MEMORY, POST_CALL, RESTORE_WINDOW],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let dplay = TestObject::dplay();
    let dp = dplay.addr();
    let all = [
        dword(DPLAY),
        dword(SESSIONS),
        dword(POSTING),
        region(SESSION_DESC, 0x50),
    ];
    let saved = Saved::new(&all);
    for (what, object, posting, result) in [
        ("no object", false, 0u32, 0u32),
        ("listed", true, 0, 0),
        ("posted", true, 1, 0),
        ("posted, cancelled", true, 1, USER_CANCEL),
        ("cancelled", true, 0, USER_CANCEL),
        ("connecting", true, 0, CONNECTING),
        ("failing", true, 0, 0x8877_0005),
    ] {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs::SENTINEL,
            &all,
            FLAGS,
            &[],
            &|| {
                p(DPLAY, if object { dp } else { 0 });
                p(SESSIONS, 5);
                p(POSTING, posting);
                fill(SESSION_DESC, 0x50, 81);
                answer(Box::new(move |place, r| {
                    if place == POST_CALL {
                        r.eax = result;
                    }
                }));
                com_answer(Box::new(move |place, _| {
                    if place == DPLAY_METHOD | ENUM_SESSIONS {
                        result
                    } else {
                        0
                    }
                }));
            },
            &find_sessions,
        );
    }
    drop(saved);
    drop(dplay);
    t.finish()
}
