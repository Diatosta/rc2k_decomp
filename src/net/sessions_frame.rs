//! The frame of `net::choose_session`, like `net::connections_frame`'s: the title (another one
//! with no session listed) and the sessions' names.

use crate::errors::Refused;
use crate::net::connections_frame::list_frame;
pub fn sessions_frame(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let vars = &o.net.vars;
    // The logic is in `rc2k_core::net::screens::sessions_title`.
    let title = rc2k_core::net::screens::sessions_title(vars.sessions.get());
    list_frame(o, title, &vars.sessions, |n| {
        crate::net::tables::with_session(vars, n, |s| s.name)
    })
}
