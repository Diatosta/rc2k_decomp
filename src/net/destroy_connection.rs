//! `net::destroy_connection` (0x4187EE): with a DirectPlay object (`dword_5AB288`), the
//! network race ended (`dword_52A910` 0), "NET: Destroy connection" logged, and through
//! DirectPlay the own player destroyed (`sub_41916B`), the session closed (`sub_418DD3`),
//! the object released (`sub_41864E`) and the lobby's (`sub_41900F`); either way the
//! network's memory given back (`net::free_messages`).
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`; the flags of `test eax, eax` are
//!      read by no caller: `lq -s 0x4187EE`)
//!
//! See `net/mod.rs`. Two callers: `net::stop` and `net::system_message` (0x31). DirectPlay's
//! four from Rust . Refuses what `net::free_messages` refuses.

use crate::errors::Refused;
use crate::logfile::log_write::log_write;
use crate::net::store::NET;
use crate::net::{close_session, destroy_player, free_messages, release_dplay, release_lobby};
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

/// The Rust version.
pub fn destroy_connection() -> Result<(), Refused> {
    if NET.dplay.get() != 0 {
        NET.session.set(0);
        unsafe {
            log_write(
                crate::pointers::host(VIEWER_PANEL.text_addr(texts::NET_DESTROY_CONNECTION))
                    as *const u8,
                true,
                2,
            )
        };
        destroy_player::destroy_player();
        close_session::close_session();
        release_dplay::release_dplay();
        release_lobby::release_lobby();
    }
    free_messages::free_messages()
}
