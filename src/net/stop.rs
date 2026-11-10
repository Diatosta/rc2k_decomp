//! `net::stop` (0x41838C): the network race ended (`dword_52A910` 0); while the receiving
//! thread is busy with a message (`dword_52A918`), a millisecond's sleep and the window's
//! messages, until it is done, or the game is closing (`g_quitting` 1) and the thread has
//! ended (`dword_52A4B0` 1); then `net::destroy_connection`.
//!
//!
//! See `net/mod.rs`. Four callers: `net::shut_down` and three of the network's menus
//! (`net::settings_link`, `net::settings_done`, `net::stopping`). It waits on another
//! thread. Refuses what `net::destroy_connection` refuses.

use crate::errors::Refused;
use crate::net::store::NetVars;
use crate::system::owner::system;
pub fn stop(vars: &NetVars) -> Result<(), Refused> {
    vars.session.set(0);
    while vars.receiving.get() != 0 {
        crate::system::sleep::sleep(1);
        crate::platform::pump_messages_now();
        if system().session.flags.quitting.get() == 1 && vars.receive_done.get() == 1 {
            break;
        }
    }
    crate::net::destroy_connection::destroy_connection(vars)
}
