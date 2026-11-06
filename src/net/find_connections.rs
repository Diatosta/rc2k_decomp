//! `net::find_connections` (0x41867C): "NET: Find connections"; unless the receiving thread
//! has ended (`g_netReceiveDone` 1): the lobby object made (`net::open_lobby`); then (the
//! original's lobby-launched branch is left out: `net::open_lobby`) DirectPlay's object made
//! (`net::create_dplay`), the lists' memory taken (`net::allocate_lists`), and the
//! connections listed (`EnumConnections` for the game's GUID `dword_5AB300` with
//! `net::connection_found`, flags DPCONNECTION_DIRECTPLAY).
//!
//! In:  nothing
//! Out: true for the carry clear (connections listed); false
//!      for the carry: the receiving thread ended, something failed or nothing was listed.
//!      The original keeps every register; its callers (the original through
//!      `net::find_connections_regs`, 0x4174BD, this inside `pusha` .. `popa`, folded in)
//!      read the carry only (`lq -s 0x4174BD`).
//! Caller: `net::find_connections_regs`.

use crate::errors::Refused;
use crate::net::lobby::log;
use crate::net::{allocate_lists, create_dplay, open_lobby};
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

/// The Rust version.
pub fn find_connections() -> Result<bool, Refused> {
    log(VIEWER_PANEL.text_ptr(texts::NET_FIND_CONNECTIONS), true, 2);
    if crate::net::owner::net().vars.receive_done.get() == 1 || !open_lobby::open_lobby()? {
        return Ok(false);
    }
    if !create_dplay::create_dplay() {
        return Ok(false);
    }
    allocate_lists::allocate_lists()?;
    crate::net::owner::net().vars.connections.set(0);
    let result =
        { crate::net::dp_objects::enum_connections(crate::net::connection_found::callback, 0) };
    // `cmp dword_52AC34, 1`: the carry when none was listed.
    Ok(result == 0 && crate::net::owner::net().vars.connections.get() >= 1)
}
