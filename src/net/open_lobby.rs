//! Makes DirectPlay's lobby object ("NET: Create dplobby"). Games launched from a lobby are
//! left out: the standalone has no lobbies (its own network protocol). Returns true when done.

use crate::errors::Refused;
use crate::net::link::Link;
use crate::net::lobby::{co_create, log_bytes};
use crate::race::viewer_screen_store::{NetTexts, texts};
pub fn open_lobby(link: &Link, lines: &NetTexts) -> Result<bool, Refused> {
    log_bytes(&lines.text(texts::NET_CREATE_DLOBBY), true, 2);
    Ok(co_create(
        &crate::net::store::CLSID_LOBBY,
        &crate::net::store::IID_LOBBY2A,
        link.lobby_cell(),
    ) == 0)
}
