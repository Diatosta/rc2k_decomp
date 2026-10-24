//! `net::find_saved_connection` (0x4185DF): the connection listed whose service provider's
//! GUID (record +0x40) is the one saved (`unk_52AC94`, `system::same_guid`).
//!
//! In:  nothing
//! Out: the connection's number (the original: eax, carry clear); None (eax -1, carry set)
//!      when none is listed or none matches. ebx, ecx kept; edx not changed by the Rust
//!      versions of its calls.
//! Callers: `net::start_link`, `net::settings_done`.

use crate::errors::Refused;
use crate::net::lobby::{CONNECTIONS, SAVED_CONNECTION, a, connection, g};
use crate::system::same_guid::same_guid;
use rc2k_core::net::lists;

/// The Rust version.
pub fn find_saved_connection() -> Result<Option<u32>, Refused> {
    // Converted: `rc2k_core::net::lists::find_listed`.
    lists::find_listed(g(CONNECTIONS), |n| -> Result<bool, Refused> {
        Ok(same_guid(
            connection(n)?.wrapping_add(0x40),
            a(SAVED_CONNECTION),
        ))
    })
}
