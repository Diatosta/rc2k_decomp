//! `net::find_connections` (0x41867C): "NET: Find connections"; unless the receiving thread
//! has ended (`g_netReceiveDone` 1): the lobby object made (`net::open_lobby`); unless a
//! lobby launched the game (its connection is made there): DirectPlay's object made
//! (`net::create_dplay`), the lists' memory taken (`net::allocate_lists`), and the
//! connections listed (`EnumConnections` for the game's GUID `dword_5AB300` with
//! `net::connection_found`, flags DPCONNECTION_DIRECTPLAY).
//!
//! In:  nothing
//! Out: true for the carry clear (connections listed, or a lobby launched the game); false
//!      for the carry: the receiving thread ended, something failed or nothing was listed.
//!      The original keeps every register; its callers (the original through
//!      `net::find_connections_regs`, 0x4174BD, this inside `pusha` .. `popa`, folded in)
//!      read the carry only (`lq -s 0x4174BD`).
//! Caller: `net::find_connections_regs`.

use crate::errors::Refused;
use crate::net::dplay::{ENUM_CONNECTIONS, call};
use crate::net::lobby::{a, log};
use crate::net::store::NET;
use crate::net::{allocate_lists, create_dplay, open_lobby};

const TEXT: usize = 0x5C2E5F; // "NET: Find connections"
const DIRECTPLAY_CONNECTIONS: u32 = 1; // DPCONNECTION_DIRECTPLAY

/// The Rust version.
pub fn find_connections() -> Result<bool, Refused> {
    log(a(TEXT), true, 2);
    if NET.receive_done.get() == 1 || !open_lobby::open_lobby()? {
        return Ok(false);
    }
    if NET.lobby_launched.get() == 1 {
        return Ok(true);
    }
    if !create_dplay::create_dplay() {
        return Ok(false);
    }
    allocate_lists::allocate_lists()?;
    NET.connections.set(0);
    let result = unsafe {
        call(
            &NET.dplay,
            ENUM_CONNECTIONS,
            &[
                (NET.app_guid.addr() as u32),
                crate::net::connection_found::callback as *const () as usize as u32,
                0,
                DIRECTPLAY_CONNECTIONS,
            ],
        )
    };
    // `cmp dword_52AC34, 1`: the carry when none was listed.
    Ok(result == 0 && NET.connections.get() >= 1)
}
