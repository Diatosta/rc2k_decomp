//! Lists the session's groups to find the host's, and adds this machine's player to it.
//! Returns true when joined.

use crate::net::create_group::join;
use crate::net::link::Link;
pub(super) fn join_group(link: &Link) -> bool {
    let result = {
        crate::net::dp_objects::enum_groups(
            link.dplay(),
            &|id, kind, short, _flags, _context| {
                crate::net::group_found::callback(link, id, kind, short)
            },
            0,
        )
    };
    result == 0 && join(link)
}
