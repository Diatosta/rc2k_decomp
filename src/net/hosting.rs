//! `net::hosting` (0x4176D5): "hosting" shown (string 0x151, as `net::connecting` shows
//! its), then the session made (`net::host_session`).
//!
//! Returns whether `net::host_session` worked.
//! Caller: `net::start_link`.

use crate::errors::Refused;
pub fn hosting(
    o: crate::owners::Owners<'static>,
    lines: &crate::race::viewer_screen_store::NetTexts,
) -> Result<bool, Refused> {
    crate::net::connecting::status(o, 0x151)?;
    crate::net::host_session::host_session(o, lines)
}
