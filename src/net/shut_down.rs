//! The network at the game's end: stops it, then closes the receiving thread's event. Refuses what `net::stop` refuses.

use crate::errors::Refused;
use std::sync::atomic::Ordering;
pub fn shut_down(
    o: crate::owners::Owners<'static>,
    lines: &crate::race::viewer_screen_store::NetTexts,
) -> Result<(), Refused> {
    let vars = &o.net.vars;
    crate::net::stop::stop(o, o.net, lines)?;
    let handle = vars.event.atomic().swap(0, Ordering::SeqCst);
    if handle != 0 {
        crate::sync::close(handle);
    }
    Ok(())
}
