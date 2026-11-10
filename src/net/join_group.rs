//! `net::join_group` (0x419354): the session's groups listed (`EnumGroups` with
//! `net::group_found`, which finds the host's "susanivanovafanclub" into `g_netSendTo`), and
//! this machine's player added to that group (`AddPlayerToGroup`).
//!
//! Returns true when joined.
//! Callers: `net::join_session`, `net::open_lobby`. DirectPlay's calls from Rust
//! (`net::dplay`).

use crate::net::create_group::join;
use crate::net::owner::net;
pub fn join_group() -> bool {
    let vars = &net().vars;
    let result = {
        crate::net::dp_objects::enum_groups(
            net().vars.dplay.get(),
            crate::net::group_found::callback,
            0,
        )
    };
    result == 0 && join(vars)
}
