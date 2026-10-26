//! `net::join_group` (0x419354): the session's groups listed (`EnumGroups` with
//! `net::group_found`, which finds the host's "susanivanovafanclub" into `g_netSendTo`), and
//! this machine's player added to that group (`AddPlayerToGroup`).
//!
//! In:  nothing
//! Out: true when joined (the original: carry clear, the flags of `test eax, eax`; every
//!      register kept), false for the carry
//! Callers: `net::join_session`, `net::open_lobby`. DirectPlay's calls from Rust
//! (`net::dplay`).

use crate::net::create_group::join;
use crate::net::dplay::{ENUM_GROUPS, call};
use crate::net::store::NET;

/// The Rust version.
pub fn join_group() -> bool {
    let result = unsafe {
        call(
            &NET.dplay,
            ENUM_GROUPS,
            &[
                0,
                crate::pointers::game_address(crate::net::group_found::callback as *const ()),
                0,
                0,
            ],
        )
    };
    result == 0 && join()
}
