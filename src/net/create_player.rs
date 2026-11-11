//! `net::create_player` (0x4190FB): "NET: Create player" and the player's name
//! (`player_name`) logged; a DPNAME made (zeroed, its size 0x10, the short
//! name) and this machine's DirectPlay player made with it (`CreatePlayer` into
//! `g_netPlayerId`, the receiving thread's event `g_netEvent`, the player's record
//! `own` as its data, 0x10 bytes, flags 0).
//!
//! Returns true when made.
//! Callers: `net::host_session`, `net::join_session`, `net::open_lobby`.
//!
//! The name goes to `net::dp_objects` as text: the DPNAME block
//! is not made (nothing else reads it).

use crate::errors::Refused;
use crate::net::lobby::log_bytes;
use crate::net::owner::Net;
use crate::race::viewer_screen_store::{ViewerPanelBlock, texts};
pub fn create_player(net: &Net, panel: &ViewerPanelBlock) -> Result<bool, Refused> {
    let vars = &net.vars;
    log_bytes(&panel.text(texts::NET_CREATE_PLAYER), false, 2);
    let name = net.setup().player_name.read();
    log_bytes(&name, true, 2);
    let made = crate::net::dp_objects::create_player(
        vars.dplay.get(),
        &name,
        vars.event.get(),
        &vars.own.bytes(),
    );
    if let Ok(id) = made {
        vars.player_id.set(id);
    }
    Ok(made.is_ok())
}
