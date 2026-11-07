//! `net::lock_received` (0x418474): the received messages' mutex (`g_netMutex`) taken:
//! `system::wait_object` (0x4110A6: `WaitForSingleObject(eax, INFINITE)`, the flags of
//! `cmp result + 1, 1` are not used).
//!
//! Six callers (`net::copy_received`, `net::start_link`, `net::lobby_frame`,
//! `net::send_chat`, `net::chat_line_locked`, `net::take_messages`). Refuses nothing.

use crate::net::store::NetVars;

/// The Rust version.
pub fn lock_received(vars: &NetVars) {
    crate::sync::wait(vars.mutex.get(), crate::sync::INFINITE);
}
