//! `net::find_sessions` (0x418999): "NET: Find sessions"; with DirectPlay's object there, the
//! sessions listed afresh (`dword_52ACF4` 0): a session description (`dword_5AB2AC`, 0x50
//! bytes, zeroed, its size 0x50, the game's GUID `dword_5AB300` at +0x18) and
//! `EnumSessions` (no timeout, `net::session_found`, flags 0x91) called, or with
//! `g_netPosting` 1 posted to the receiving thread (`net::post_call`, the code at 0x418A43);
//! then the window restored (`system::restore_window`).
//!
//! In:  nothing
//! Out: true for the carry clear (done, or DPERR_CONNECTING); false for the carry: no object,
//!      the user cancelled (DPERR_USERCANCEL) or it failed. The original keeps every
//!      register; its callers (the original through `net::find_sessions_regs`, 0x41771E,
//!      this inside `pusha` .. `popa`, folded in) read the carry only (`lq -s 0x41771E`).
//! Caller: `net::find_sessions_regs`.

use crate::errors::Refused;
use crate::net::dplay::{ENUM_SESSIONS, call};
use crate::net::lobby::{
    APP_GUID, DPLAY, POSTING, SESSION_DESC, SESSIONS, a, g, log, movsb, p, zero,
};

const TEXT: usize = 0x5C2EC7; // "NET: Find sessions"
const ENUM_FLAGS: u32 = 0x91;
const CONNECTING: u32 = 0x8877_015E; // DPERR_CONNECTING

/// The session description made ready: zeroed, its size.
pub fn zeroed_description() -> Result<(), Refused> {
    zero(a(SESSION_DESC), 0x50)?;
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

/// The piece `code` made: posted to the receiving thread (`net::post_call`) while
/// `g_netPosting` is 1, else called here.
pub fn posted_or_called(code: extern "C" fn() -> u32) -> u32 {
    if g(POSTING) == 1 {
        crate::net::post_call::post_call(code as *const () as u32)
    } else {
        code()
    }
}

/// The Rust version.
pub fn find_sessions() -> Result<bool, Refused> {
    log(a(TEXT), true, 2);
    if g(DPLAY) == 0 {
        return Ok(false);
    }
    p(SESSIONS, 0);
    zeroed_description()?;
    movsb(a(SESSION_DESC) + 0x18, a(APP_GUID), 0x10);
    let result = posted_or_called(posted_enum_sessions);
    crate::system::restore_window::restore_window();
    Ok(result == 0 || result == CONNECTING)
}
