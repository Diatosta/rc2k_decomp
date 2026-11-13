//! Takes the received messages' mutex.

use crate::net::store::NetShared;
pub fn lock_received(vars: &NetShared) {
    crate::sync::wait(vars.mutex.get(), crate::sync::INFINITE);
}
