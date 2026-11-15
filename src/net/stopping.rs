//! With a session on, shows "stopping" on the background, then stops the network.

use crate::errors::Refused;
pub fn stopping(
    o: crate::owners::Owners<'static>,
    lines: &crate::race::viewer_screen_store::NetTexts,
) -> Result<(), Refused> {
    if o.net.race_slot.is_on() {
        crate::net::show_background::show_background(o)?;
        crate::net::connecting::status(o, 0x157)?;
    }
    crate::net::stop::stop(o, o.net, lines)
}
