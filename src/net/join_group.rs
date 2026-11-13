//! Lists the session's groups to find the host's, and adds this machine's player to it.
//! Returns true when joined.

use crate::net::create_group::join;
use crate::net::store::NetShared;
pub fn join_group(vars: &NetShared) -> bool {
    let result = {
        crate::net::dp_objects::enum_groups(
            vars.dplay.get(),
            &|id, kind, short, _flags, _context| {
                crate::net::group_found::callback(vars, id, kind, short)
            },
            0,
        )
    };
    result == 0 && join(vars)
}
