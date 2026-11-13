//! `net::shut_down` (0x4184D6): the network at the game's end: `net::stop`, then the
//! received messages' mutex (`mutex`) and the receiving thread's event
//! (`event`) taken out (`xchg` with 0) and closed (0x411092, 0x41104F:
//! `CloseHandle` if not 0).
//!
//! See `net/mod.rs`. One caller in the standalone, `start`'s way out (0x4010EE; the
//! original's other, 0x4104F2, is not reached). Refuses what `net::stop` refuses.

use crate::errors::Refused;
use std::sync::atomic::Ordering;
pub fn shut_down(
    o: crate::owners::Owners<'static>,
    lines: &crate::race::viewer_screen_store::NetTexts,
) -> Result<(), Refused> {
    let vars = &o.net.vars;
    crate::net::stop::stop(o, vars, lines)?;
    for cell in [&vars.mutex, &vars.event] {
        let handle = cell.atomic().swap(0, Ordering::SeqCst);
        if handle != 0 {
            crate::sync::close(handle);
        }
    }
    Ok(())
}
