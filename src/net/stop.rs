//! `net::stop` (0x41838C): the network race ended (`session` 0); while the receiving
//! thread is busy with a message (`receiving`), a millisecond's sleep and the window's
//! messages, until it is done, or the game is closing (`g_quitting` 1) and the thread has
//! ended (`receive_done` 1); then `net::destroy_connection`.
//!
//!
//! See `net/mod.rs`. Four callers: `net::shut_down` and three of the network's menus
//! (`net::settings_link`, `net::settings_done`, `net::stopping`). It waits on another
//! thread. Refuses what `net::destroy_connection` refuses.

use crate::errors::Refused;
use crate::net::store::NetShared;
use crate::race::viewer_screen_store::NetTexts;
pub fn stop(
    o: crate::owners::Owners<'static>,
    vars: &NetShared,
    lines: &NetTexts,
) -> Result<(), Refused> {
    vars.session.set(0);
    while vars.receiving.get() != 0 {
        crate::system::sleep::sleep(1);
        crate::platform::pump_messages_now(o);
        if o.system.session.flags.quitting.is_set() && vars.receive_done.get() == 1 {
            break;
        }
    }
    crate::net::destroy_connection::destroy_connection(vars, lines)
}
