//! Releases the lobby object, if any.

use crate::net::link::Link;
pub fn release_lobby(link: &Link) {
    let _ = link.release_lobby();
}
