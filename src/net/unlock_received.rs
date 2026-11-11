//! `net::unlock_received` (0x418481): the received messages' mutex (`g_netMutex`) given
//! back: `system::release_mutex` (0x4110B9: `ReleaseMutex(handle)`, whose flags
//! are not used).
//!
//! Six callers, those of `net::lock_received`. Refuses nothing.

use crate::net::store::NetVars;
pub fn unlock_received(vars: &NetVars) {
    crate::sync::release_mutex(vars.mutex.get());
}
