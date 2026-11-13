//! Ends the network race; while the receiving thread is busy with a message, sleeps a
//! millisecond and pumps the window's messages until it is done (or the game is closing and the
//! thread has ended); then destroys the connection. Refuses what `net::destroy_connection`
//! refuses.

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
