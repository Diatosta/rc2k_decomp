//! `net::unlock_received` (0x418481): the received messages' mutex (`g_netMutex`) given
//! back: `system::release_mutex` (0x4110B9: `ReleaseMutex(eax)`, the flags of
//! `cmp result, 1` are not used).
//!
//! Six callers, those of `net::lock_received`. Refuses nothing.

use crate::net::store::NetVars;

/// The Rust version.
pub fn unlock_received(vars: &NetVars) {
    crate::sync::release_mutex(vars.mutex.get());
}
