//! `net::shut_down` (0x4184D6): the network at the game's end: `net::stop`, then the
//! received messages' mutex (`mutex`) and the receiving thread's event
//! (`event`) taken out (`xchg` with 0) and closed (0x411092, 0x41104F:
//! `CloseHandle` if not 0).
//!
//! See `net/mod.rs`. One caller in the standalone, `start`'s way out (0x4010EE; the
//! original's other, 0x4104F2, is not reached). Refuses what `net::stop` refuses.

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::system::owner::System;
use std::sync::atomic::Ordering;
pub fn shut_down(
    net: &Net,
    system: &System,
    panel: &crate::race::viewer_screen_store::ViewerPanelBlock,
) -> Result<(), Refused> {
    let vars = &net.vars;
    crate::net::stop::stop(vars, system, panel)?;
    for cell in [&vars.mutex, &vars.event] {
        let handle = cell.atomic().swap(0, Ordering::SeqCst);
        if handle != 0 {
            crate::sync::close(handle);
        }
    }
    Ok(())
}
