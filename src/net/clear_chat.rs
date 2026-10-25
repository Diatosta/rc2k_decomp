//! `net::clear_chat` (0x419411): the lobby's chat (`byte_52A508`) made eight empty lines:
//! seven 0x0A and a NUL.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! Caller: `net::start_link` (with the received messages' lock held).

use crate::net::store::NET;

/// The Rust version.
pub fn clear_chat() {
    // Converted: `rc2k_core::net::chat::EMPTY`.
    unsafe {
        ((NET.chat.addr() as u32) as *mut [u8; 8]).write_unaligned(rc2k_core::net::chat::EMPTY)
    };
}
