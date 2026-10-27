//! `net::keep_connection` (0x417661): connection eax kept as the one to use next time: its
//! service provider's GUID (record +0x40, 16 bytes) to `unk_52AC94`, its name (64 bytes)
//! to `unk_52ACA4`.
//!
//! In:  `connection` (eax: the connection's number)
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! Callers: `net::start_link`, `net::settings_link`, `net::settings_done`. Refuses what
//! `net::lobby::connection` refuses (its fatal errors).

use crate::errors::Refused;
use crate::net::store::NET;
use crate::net::tables::with_connection;
use crate::pointers::GamePtr;

/// The Rust version.
pub fn keep_connection(number: u32) -> Result<(), Refused> {
    let (provider, name) = with_connection(number, |c| (c.provider, c.name))?;
    GamePtr::new(NET.saved_connection.addr()).write_bytes(&provider, "the saved connection")?;
    GamePtr::new(NET.saved_name.addr()).write_bytes(&name, "the saved connection")?;
    Ok(())
}
