//! `net::joining` (0x4179B6): "joining" shown (string 0x153, as `net::connecting` shows
//! its), then the session joined (`net::join_session`).
//!
//! Takes `session`. Returns whether `net::join_session` worked.
//! Caller: `net::start_link`.

use crate::errors::Refused;
use crate::net::owner::Net;
pub fn joining(
    net: &Net,
    system: &crate::system::owner::System,
    panel: &crate::race::viewer_screen_store::ViewerPanelBlock,
    session: u32,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<bool, Refused> {
    crate::net::connecting::status(0x153, gfx, system)?;
    crate::net::join_session::join_session(net, system, panel, session)
}
