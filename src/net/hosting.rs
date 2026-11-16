//! Shows "hosting", then makes the session with `net::host_session`. Returns whether that
//! worked.

use crate::errors::Refused;
pub fn hosting(
    o: crate::owners::Owners<'static>,
    lines: &crate::race::viewer_screen_store::NetTexts,
) -> Result<bool, Refused> {
    crate::net::connecting::status(o, 0x151)?;
    crate::net::host_session::host_session(o, lines)
}
