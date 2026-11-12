//! `net::sessions_frame` (0x41790E): the frame of `net::choose_session`, as
//! `net::connections_frame`'s: the title string 0x150 (0x152 with no session listed), the
//! sessions' names (`net::session_entry`).
//!
//! Called by `screens::run_frame` for `net::choose_session` (`routes.rs`).
//! Called by `screens::run_frame` for `net::choose_session`.

use crate::errors::Refused;
use crate::install::owner::Install;
use crate::net::connections_frame::list_frame;
use crate::net::owner::Net;
pub fn sessions_frame(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    net: &Net,
    install: &Install,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let vars = &net.vars;
    // Converted: `rc2k_core::net::screens::sessions_title`.
    let title = rc2k_core::net::screens::sessions_title(vars.sessions.get());
    list_frame(
        so,
        install,
        title,
        &vars.sessions,
        |n| crate::net::tables::with_session(vars, n, |s| s.name),
        gfx,
        system,
    )
}
