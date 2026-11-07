//! `net::destroy_connection` (0x4187EE): with a DirectPlay object (`dword_5AB288`), the
//! network race ended (`dword_52A910` 0), "NET: Destroy connection" logged, and through
//! DirectPlay the own player destroyed (`sub_41916B`), the session closed (`sub_418DD3`),
//! the object released (`sub_41864E`) and the lobby's (`sub_41900F`); either way the
//! network's memory given back (`net::free_messages`).
//!
//!
//! See `net/mod.rs`. Two callers: `net::stop` and `net::system_message` (0x31). DirectPlay's
//! four from Rust . Refuses what `net::free_messages` refuses.

use crate::errors::Refused;
use crate::net::store::NetVars;
use crate::net::{close_session, destroy_player, free_messages, release_dplay, release_lobby};
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

/// The Rust version.
pub fn destroy_connection(vars: &NetVars) -> Result<(), Refused> {
    if vars.dplay.get() != 0 {
        vars.session.set(0);
        crate::logfile::log_write::log_at(
            VIEWER_PANEL.text_ptr(texts::NET_DESTROY_CONNECTION),
            true,
            2,
        );
        destroy_player::destroy_player(vars);
        close_session::close_session(vars);
        release_dplay::release_dplay(vars);
        release_lobby::release_lobby(vars);
    }
    free_messages::free_messages()
}
