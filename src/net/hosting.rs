//! `net::hosting` (0x4176D5): "hosting" shown (string 0x151, as `net::connecting` shows
//! its), then the session made (`net::host_session`).
//!
//! Returns whether `net::host_session` worked.
//! Caller: `net::start_link`.

use crate::errors::Refused;
use crate::net::owner::Net;
pub fn hosting(
    net: &Net,
    system: &crate::system::owner::System,
    panel: &crate::race::viewer_screen_store::ViewerPanelBlock,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<bool, Refused> {
    crate::net::connecting::status(0x151, gfx, system)?;
    crate::net::host_session::host_session(net, system, panel)
}
