//! `net::settings_link` (0x416EEC): Enter on the settings screen's connection: the
//! connections listed (`net::find_connections_regs`), one chosen (`net::choose_connection`)
//! and kept (`net::keep_connection`); then the network stopped again (`net::stop`).
//!
//! Caller: `net::settings_loop`. Long-running (a list screen).

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::race::viewer_screen_store::ViewerPanelBlock;
use crate::system::owner::System;
pub fn settings_link(
    install: &'static crate::install::owner::Install,
    net: &Net,
    system: &System,
    panel: &ViewerPanelBlock,
) -> Result<(), Refused> {
    if crate::net::find_connections::find_connections(&net.vars, panel)? {
        let chosen = crate::net::choose_connection::choose_connection(install, net, system)?;
        if (chosen as i32) >= 0 {
            crate::net::keep_connection::keep_connection(net, chosen)?;
        }
    }
    crate::net::stop::stop(&net.vars, system)
}
