//! `net::stopping` (0x417089): with the session on (`g_netSession` 1), the background
//! (`net::show_background`) and "stopping" shown (string 0x157, as `net::connecting` shows
//! its); then the network stopped (`net::stop`).
//!
//! Caller: `net::start_link`.

use crate::errors::Refused;
pub fn stopping(
    o: crate::owners::Owners<'static>,
    lines: &crate::race::viewer_screen_store::NetTexts,
) -> Result<(), Refused> {
    let vars = &o.net.vars;
    if vars.session.get() == 1 {
        crate::net::show_background::show_background(o)?;
        crate::net::connecting::status(o, 0x157)?;
    }
    crate::net::stop::stop(o, vars, lines)
}
