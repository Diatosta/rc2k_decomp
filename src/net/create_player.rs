//! Makes this machine's DirectPlay player ("NET: Create player"), with the receiving thread's
//! event and the player's record as its data. The name goes to `net::dp_objects` as text.
//! Returns true when made.

use crate::errors::Refused;
use crate::net::lobby::log_bytes;
use crate::net::owner::Net;
use crate::race::viewer_screen_store::{NetTexts, texts};
pub(super) fn create_player(net: &Net, lines: &NetTexts) -> Result<bool, Refused> {
    let (vars, link) = (&net.vars, &net.link);
    log_bytes(&lines.text(texts::NET_CREATE_PLAYER), false, 2);
    let name = net.setup().player_name.read();
    log_bytes(&name, true, 2);
    let made = crate::net::dp_objects::create_player(
        link.dplay(),
        &name,
        vars.event.get(),
        &vars.own.bytes(),
    );
    if let Ok(id) = made {
        link.set_player_id(id);
    }
    Ok(made.is_ok())
}
