//! Steam is not part of this build (a target without it): it is never available, and the
//! connection list does not show it.

use crate::net::dp_session::{Desc, Session, Wire};
use std::sync::Arc;

/// Whether Steam can be used: never here.
pub fn available() -> bool {
    false
}

/// Makes the session's lobby and listener: never reached, Steam being unavailable.
pub fn host(_session: &Arc<Session>, _desc: &Desc, _password: bool) -> Result<(), String> {
    Err("this build has no Steam".into())
}

/// The sessions found in Steam: none.
pub fn lobbies(_application: [u8; 16]) -> Vec<Desc> {
    vec![]
}

/// A connection to the host of the lobby `instance` stands for: never reached.
pub fn join(_instance: &[u8; 16]) -> Result<Box<dyn Wire>, u32> {
    Err(crate::net::dp_session::DPERR_NOCONNECTION)
}
