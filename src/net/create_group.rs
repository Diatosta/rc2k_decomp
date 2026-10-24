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
use crate::net::lobby::{DP_NAME, DPLAY, PLAYER_ID, SEND_TO, a, g};

pub const GROUP_NAME: usize = 0x5AB25C; // "susanivanovafanclub"

/// `AddPlayerToGroup(g_netSendTo, g_netPlayerId)`: whether it worked; shared with
/// `net::join_group`.
pub fn join() -> bool {
    unsafe { call(DPLAY, ADD_PLAYER_TO_GROUP, &[g(SEND_TO), g(PLAYER_ID)]) == 0 }
}

/// The Rust version.
pub fn create_group() -> Result<bool, Refused> {
    dp_name(a(GROUP_NAME))?;
    let result = unsafe { call(DPLAY, CREATE_GROUP, &[a(SEND_TO), a(DP_NAME), 0, 0, 0]) };
    Ok(result == 0 && join())
}
