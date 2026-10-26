//! `net::joining` (0x4179B6): "joining" shown (string 0x153, as `net::connecting` shows
//! its), then session eax joined (`net::join_session`).
//!
//! In:  `session` (eax: the session's number)
//! Out: whether `net::join_session` worked (the original: its flags, carry when it failed;
//!      every register kept)
//! Caller: `net::start_link`.

use crate::errors::Refused;

/// The Rust version.
pub fn joining(session: u32) -> Result<bool, Refused> {
    crate::net::connecting::status(0x153)?;
    crate::net::join_session::join_session(session)
}
