//! The frame of `net::choose_session`, like `net::connections_frame`'s: the title (another one
//! with no session listed) and the sessions' names.

use crate::errors::Refused;
use crate::net::connections_frame::list_frame;
pub fn sessions_frame(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let link = &o.net.link;
    // The logic is in `rc2k_core::net::screens::sessions_title`.
    let title = rc2k_core::net::screens::sessions_title(link.sessions().get());
    list_frame(o, title, link.sessions(), |n| {
        crate::net::tables::with_session(link, n, |s| s.name)
    })
}
