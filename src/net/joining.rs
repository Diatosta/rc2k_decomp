//! `net::joining` (0x4179B6): "joining" shown (string 0x153, as `net::connecting` shows
//! its), then the session joined (`net::join_session`).
//!
//! Takes `session`. Returns whether `net::join_session` worked.
//! Caller: `net::start_link`.

use crate::errors::Refused;
use crate::net::owner::Net;
pub fn joining(net: &Net, session: u32) -> Result<bool, Refused> {
    crate::net::connecting::status(0x153)?;
    crate::net::join_session::join_session(net, session)
}
