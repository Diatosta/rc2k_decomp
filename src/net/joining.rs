//! Shows "joining", then joins the session with `net::join_session`. Returns whether that
//! worked.

use crate::errors::Refused;
pub fn joining(
    o: crate::owners::Owners<'static>,
    lines: &crate::race::viewer_screen_store::NetTexts,
    session: u32,
) -> Result<bool, Refused> {
    crate::net::connecting::status(o, 0x153)?;
    crate::net::join_session::join_session(o, lines, session)
}
