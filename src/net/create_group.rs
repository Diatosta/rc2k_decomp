//! `net::create_group` (0x4192EA): the host's group made, which every player joins and every
//! message goes to: a DPNAME (`dp_name`) named "susanivanovafanclub" (`0x5AB25C`),
//! `CreateGroup` into `g_netSendTo` (no data, flags 0), and this machine's player added to
//! it (`AddPlayerToGroup`).
//!
//! Returns true when made and joined.
//! Callers: `net::host_session`, `net::open_lobby`.
//!
//! The name goes to `net::dp_objects` as text: the DPNAME is not made.

use crate::errors::Refused;
use crate::net::store::NetVars;

/// The player put in the group (`AddPlayerToGroup`): whether it went.
pub fn join(vars: &NetVars) -> bool {
    crate::net::dp_objects::add_player_to_group(
        vars.dplay.get(),
        vars.send_to.get(),
        vars.player_id.get(),
    ) == 0
}
pub fn create_group(vars: &NetVars) -> Result<bool, Refused> {
    let made = crate::net::dp_objects::create_group(vars.dplay.get(), &vars.group_name.read(), &[]);
    if let Ok(id) = made {
        vars.send_to.set(id);
    }
    Ok(made.is_ok() && join(vars))
}
