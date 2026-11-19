//! Ends the network race; while the receiving thread is busy with a message, sleeps a
//! millisecond and pumps the window's messages until it is done (or the game is closing and the
//! thread has ended); then destroys the connection. Refuses what `net::destroy_connection`
//! refuses.

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::race::viewer_screen_store::NetTexts;
pub fn stop(o: crate::owners::Owners<'static>, net: &Net, lines: &NetTexts) -> Result<(), Refused> {
    let vars = &net.vars;
    net.race_slot.end();
    while vars.receiver.receiving.get() != 0 {
        crate::clock::sleep::sleep(1);
        crate::platform::pump_messages_now(o);
        if o.system.session.flags.quitting.is_set() && vars.receive_done.get() == 1 {
            break;
        }
    }
    crate::net::destroy_connection::destroy_connection(net, lines)
}
