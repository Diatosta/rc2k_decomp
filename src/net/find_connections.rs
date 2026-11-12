//! `net::find_connections` (0x41867C): "NET: Find connections"; unless the receiving thread
//! has ended (`g_netReceiveDone` 1): the lobby object made (`net::open_lobby`); then (the
//! original's lobby-launched branch is left out: `net::open_lobby`) DirectPlay's object made
//! (`net::create_dplay`), the lists' memory taken (`net::allocate_lists`), and the
//! connections listed (`EnumConnections` for the game's GUID `app_guid` with
//! `net::connection_found`, flags DPCONNECTION_DIRECTPLAY).
//!
//! Returns true when connections were listed; false when the receiving thread ended, something
//! failed or nothing was listed. The original's callers read only the carry (`lq -s 0x4174BD`).
//! Caller: `net::find_connections_regs`.

use crate::errors::Refused;
use crate::net::lobby::log_bytes;
use crate::net::store::NetVars;
use crate::net::{allocate_lists, create_dplay, open_lobby};
use crate::race::viewer_screen_store::{ViewerPanelBlock, texts};
pub fn find_connections(
    vars: &NetVars,
    strings: &crate::strings::owner::Strings,
    panel: &ViewerPanelBlock,
) -> Result<bool, Refused> {
    log_bytes(&panel.text(texts::NET_FIND_CONNECTIONS), true, 2);
    if vars.receive_done.get() == 1 || !open_lobby::open_lobby(vars, panel)? {
        return Ok(false);
    }
    if !create_dplay::create_dplay(vars, panel) {
        return Ok(false);
    }
    allocate_lists::allocate_lists(vars)?;
    vars.connections.set(0);
    let result = crate::net::dp_objects::enum_connections(
        &|guid, data, short, _flags, _context| {
            crate::net::connection_found::callback(vars, strings, panel, guid, data, short)
        },
        0,
    );
    // The carry is set when none was listed.
    Ok(result == 0 && vars.connections.get() >= 1)
}
