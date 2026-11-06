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
use crate::net::lobby::log;
use crate::net::owner::Net;
use crate::net::owner::net;
use crate::net::store::NetVars;
use crate::net::store::put_bytes;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

const ENUM_FLAGS: u32 = 0x91;
const CONNECTING: u32 = 0x8877_015E; // DPERR_CONNECTING

/// The session description made ready: zeroed, its size.
pub fn zeroed_description(vars: &NetVars) -> Result<(), Refused> {
    vars.session_desc.clear();
    vars.session_desc.size.set(0x50);
    Ok(())
}

/// `EnumSessions` as the code at 0x418A43 calls it.
pub fn enum_sessions() -> u32 {
    let vars = &net().vars;
    let application = crate::net::dp_objects::desc(&vars.session_desc, b"").application;
    crate::net::dp_objects::enum_sessions(
        vars.dplay.get(),
        &application,
        crate::net::session_found::callback,
        0,
        ENUM_FLAGS,
    )
}

/// The code at 0x418A43 as the receiving thread calls it when it is posted.
pub fn posted_enum_sessions() -> u32 {
    enum_sessions()
}

/// The piece `code` made: posted to the receiving thread (`net::post_call`) while
/// `g_netPosting` is 1, else called here.
pub fn posted_or_called(net: &Net, code: fn() -> u32) -> u32 {
    if net.vars.posting.get() == 1 {
        crate::net::post_call::post_call(net, code)
    } else {
        code()
    }
}

/// The Rust version.
pub fn find_sessions(net: &Net) -> Result<bool, Refused> {
    let vars = &net.vars;
    log(VIEWER_PANEL.text_ptr(texts::NET_FIND_SESSIONS), true, 2);
    if vars.dplay.get() == 0 {
        return Ok(false);
    }
    vars.sessions.set(0);
    zeroed_description(&net.vars)?;
    put_bytes(&vars.session_desc.application, &vars.app_guid.read());
    let result = posted_or_called(net, posted_enum_sessions);
    crate::system::restore_window::restore_window();
    Ok(result == 0 || result == CONNECTING)
}
