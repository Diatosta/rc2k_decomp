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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::dplay::{ENUM_SESSIONS, call};
use crate::net::lobby::{
    APP_GUID, DPLAY, LOG_L2_NL, POST_CALL, POSTING, Part, RESTORE_WINDOW, SESSION_DESC, SESSIONS,
    ZERO_MEMORY, a, cmp_flags, g, game, lobby_entry, log_text, movsb, p, test_eax,
};

const TEXT: usize = 0x5C2EC7; // "NET: Find sessions"
pub const POSTED_CODE: u32 = 0x418A43; // the piece posted: `EnumSessions`
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
            &[
                a(SESSION_DESC),
                0,
                crate::net::session_found::callback as *const () as usize as u32,
                0,
                ENUM_FLAGS,
            ],
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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, find_sessions)
}
