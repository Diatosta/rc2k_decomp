//! `net::unlock_received` (0x418481): the received messages' mutex (`g_netMutex`) given
//! back: `system::release_mutex` (0x4110B9: `ReleaseMutex(eax)`, the flags of
//! `cmp result, 1`) inside `pusha` .. `popa`.
//!
//! In:  nothing
//! Out: nothing (the original keeps every register; its flags are read by no caller:
//!      `lq -s 0x418481`)
//! Six callers, those of `net::lock_received`. Refuses nothing.

use crate::net::{MUTEX, g};

/// The Rust version.
pub fn unlock_received() {
    unsafe { windows_sys::Win32::System::Threading::ReleaseMutex(g(MUTEX) as _) };
}
