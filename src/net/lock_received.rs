//! `net::lock_received` (0x418474): the received messages' mutex (`g_netMutex`) taken:
//! `system::wait_object` (0x4110A6: `WaitForSingleObject(eax, INFINITE)`, the flags of
//! `cmp result + 1, 1`) inside `pusha` .. `popa`.
//!
//! In:  nothing
//! Out: nothing (the original keeps every register; its flags are read by no caller:
//!      `lq -s 0x418474`, every site's live set holds DF only of the flags)
//! Six callers (`net::copy_received`, `net::start_link`, `net::lobby_frame`,
//! `net::send_chat`, `net::chat_line_locked`, `net::take_messages`). Refuses nothing.

use crate::net::store::NET;

/// The Rust version.
pub fn lock_received() {
    unsafe {
        windows_sys::Win32::System::Threading::WaitForSingleObject(
            crate::pointers::host(NET.mutex.get()) as _,
            u32::MAX,
        )
    };
}
