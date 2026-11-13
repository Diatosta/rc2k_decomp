//! Gives the received messages' mutex back.

use crate::net::store::NetShared;
pub fn unlock_received(vars: &NetShared) {
    crate::sync::release_mutex(vars.mutex.get());
}
