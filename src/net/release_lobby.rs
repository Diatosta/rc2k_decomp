//! Releases the lobby object, if any.

use crate::net::store::NetShared;
pub fn release_lobby(vars: &NetShared) {
    if vars.lobby.get() != 0 {
        crate::net::dp_objects::release(vars.lobby.get());
        vars.lobby.set(0);
    }
}
