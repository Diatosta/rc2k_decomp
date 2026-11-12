//! `net::stopping` (0x417089): with the session on (`g_netSession` 1), the background
//! (`net::show_background`) and "stopping" shown (string 0x157, as `net::connecting` shows
//! its); then the network stopped (`net::stop`).
//!
//! Caller: `net::start_link`.

use crate::errors::Refused;
use crate::net::store::NetVars;
pub fn stopping(
    vars: &NetVars,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
    panel: &crate::race::viewer_screen_store::ViewerPanelBlock,
) -> Result<(), Refused> {
    if vars.session.get() == 1 {
        crate::net::show_background::show_background(gfx, system, vars)?;
        crate::net::connecting::status(0x157, gfx)?;
    }
    crate::net::stop::stop(vars, system, panel)
}
