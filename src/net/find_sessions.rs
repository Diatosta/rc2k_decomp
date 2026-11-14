//! Lists the sessions afresh ("NET: Find sessions") with `EnumSessions` for the game's GUID,
//! called directly or, with `posting` 1, on the receiving thread (`net::post_call`); then the
//! window is restored. Returns true when done (or DPERR_CONNECTING); false for no object, a
//! user cancel or a failure.

use crate::errors::Refused;
use crate::net::lobby::log_bytes;
use crate::net::owner::Net;
use crate::net::store::NetShared;
use crate::net::store::put_bytes;
use crate::race::viewer_screen_store::{NetTexts, texts};

const ENUM_FLAGS: u32 = 0x91;
const CONNECTING: u32 = 0x8877_015E; // DPERR_CONNECTING

/// The session description made ready: zeroed, its size.
pub fn zeroed_description(vars: &NetShared) -> Result<(), Refused> {
    vars.session_desc.clear();
    vars.session_desc.size.set(0x50);
    Ok(())
}

/// `EnumSessions` as the receiving thread calls it when it is posted.
pub fn enum_sessions(net: &Net, lines: &NetTexts) -> u32 {
    let vars = &net.vars;
    let application = crate::net::dp_objects::desc(&vars.session_desc, b"").application;
    crate::net::dp_objects::enum_sessions(
        vars.dplay.get(),
        &application,
        &|desc, flags, _context| crate::net::session_found::callback(vars, lines, desc, flags),
        0,
        ENUM_FLAGS,
    )
}

/// The enumeration as the receiving thread calls it when it is posted.
pub fn posted_enum_sessions(net: &Net, lines: &NetTexts) -> u32 {
    enum_sessions(net, lines)
}

/// The piece `code` made: posted to the receiving thread (`net::post_call`) while `posting`
/// is 1, else called here.
pub fn posted_or_called(
    o: crate::owners::Owners<'static>,
    lines: &NetTexts,
    code: crate::net::post_call::PostedCall,
) -> u32 {
    if o.net.vars.receiver.posting.get() == 1 {
        crate::net::post_call::post_call(o, code)
    } else {
        code(o.net, lines)
    }
}
pub fn find_sessions(o: crate::owners::Owners<'static>, lines: &NetTexts) -> Result<bool, Refused> {
    let net = o.net;
    let vars = &net.vars;
    log_bytes(&lines.text(texts::NET_FIND_SESSIONS), true, 2);
    if vars.dplay.get() == 0 {
        return Ok(false);
    }
    vars.sessions.set(0);
    zeroed_description(&net.vars)?;
    put_bytes(&vars.session_desc.application, &crate::net::store::APP_GUID);
    let result = posted_or_called(o, lines, posted_enum_sessions);
    crate::system::restore_window::restore_window();
    Ok(result == 0 || result == CONNECTING)
}
