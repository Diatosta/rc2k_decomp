//! `net::create_group` (0x4192EA): the host's group made, which every player joins and every
//! message goes to: a DPNAME (`dword_5AB29C`) named "susanivanovafanclub" (`0x5AB25C`),
//! `CreateGroup` into `g_netSendTo` (no data, flags 0), and this machine's player added to
//! it (`AddPlayerToGroup`).
//!
//! In:  nothing
//! Out: true when made and joined (the original: carry clear, the flags of `test eax,
//!      eax`; every register kept), false for the carry
//! Callers: `net::host_session`, `net::open_lobby`.
//!
//! Since 2026-10-29 the name goes to `net::dp_objects` as text: the DPNAME is not made.

use crate::errors::Refused;
use crate::net::store::NET;

/// The player put in the group (`AddPlayerToGroup`): whether it went.
pub fn join() -> bool {
    crate::net::dp_objects::add_player_to_group(
        NET.dplay.get(),
        NET.send_to.get(),
        NET.player_id.get(),
    ) == 0
}

/// The Rust version (see the header).
pub fn create_group() -> Result<bool, Refused> {
    let made = crate::net::dp_objects::create_group(NET.dplay.get(), &NET.group_name.read(), &[]);
    if let Ok(id) = made {
        NET.send_to.set(id);
    }
    Ok(made.is_ok() && join())
}
