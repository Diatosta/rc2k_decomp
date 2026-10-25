//! `net::create_group` (0x4192EA): the host's group made, which every player joins and every
//! message goes to: a DPNAME (`dword_5AB29C`) named "susanivanovafanclub" (`0x5AB25C`),
//! `CreateGroup` into `g_netSendTo` (no data, flags 0), and this machine's player added to
//! it (`AddPlayerToGroup`).
//!
//! In:  nothing
//! Out: true when made and joined (the original: carry clear, the flags of `test eax,
//!      eax`; every register kept), false for the carry
//! Callers: `net::host_session`, `net::open_lobby`.

use crate::errors::Refused;
use crate::net::create_player::dp_name;
use crate::net::dplay::{ADD_PLAYER_TO_GROUP, CREATE_GROUP, call};
use crate::net::store::NET;

/// `AddPlayerToGroup(g_netSendTo, g_netPlayerId)`: whether it worked; shared with
/// `net::join_group`.
pub fn join() -> bool {
    unsafe {
        call(
            &NET.dplay,
            ADD_PLAYER_TO_GROUP,
            &[NET.send_to.get(), NET.player_id.get()],
        ) == 0
    }
}

/// The Rust version.
pub fn create_group() -> Result<bool, Refused> {
    dp_name(NET.group_name.addr() as u32)?;
    let result = unsafe {
        call(
            &NET.dplay,
            CREATE_GROUP,
            &[
                NET.send_to.addr() as u32,
                NET.dp_name.addr() as u32,
                0,
                0,
                0,
            ],
        )
    };
    Ok(result == 0 && join())
}
