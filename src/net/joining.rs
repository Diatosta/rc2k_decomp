//! `net::joining` (0x4179B6): "joining" shown (string 0x153, as `net::connecting` shows
//! its), then the session joined (`net::join_session`).
//!
//! Takes `session`. Returns whether `net::join_session` worked.
//! Caller: `net::start_link`.

use crate::errors::Refused;
pub fn joining(
    o: crate::owners::Owners<'static>,
    lines: &crate::race::viewer_screen_store::NetTexts,
    session: u32,
) -> Result<bool, Refused> {
    crate::net::connecting::status(o, 0x153)?;
    crate::net::join_session::join_session(o, lines, session)
}
