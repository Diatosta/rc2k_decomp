//! `net::open_lobby` (0x418E9B): "NET: Create dplobby", DirectPlay's lobby object made
//! (`CoCreateInstance` of CLSID_DirectPlayLobby `clsid_lobby`, IID_IDirectPlayLobby2A
//! `iid_lobby2a`) into `g_dplayLobby`.
//!
//! Returns true when done.
//! Left out: the original's branch for a game a lobby launched (`g_lobbyLaunched` 1: the
//! connection settings read, the lobby's `Connect`, the player and the session made from
//! them): the standalone has no lobbies (its own network protocol), so `net::set_up` leaves out the
//! original's lobby detection (0x418E23) and `g_lobbyLaunched` stays 0. (Its
//! `Connect` could not have worked in the original either: section 16.)
//! Caller: `net::find_connections`.

use crate::errors::Refused;
use crate::net::lobby::{co_create, log_bytes};
use crate::net::store::NetVars;
use crate::race::viewer_screen_store::{ViewerPanelBlock, texts};
pub fn open_lobby(vars: &NetVars, panel: &ViewerPanelBlock) -> Result<bool, Refused> {
    log_bytes(&panel.text(texts::NET_CREATE_DLOBBY), true, 2);
    Ok(co_create(&vars.clsid_lobby, &vars.iid_lobby2a, &vars.lobby) == 0)
}
