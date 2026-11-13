//! The host's group, which every player joins and every message goes to, made under the name
//! "susanivanovafanclub" and the own player added to it. The name goes to `net::dp_objects` as
//! text. Returns true when made and joined.

use crate::errors::Refused;
use crate::net::store::NetShared;

/// The player put in the group (`AddPlayerToGroup`): whether it went.
pub fn join(vars: &NetShared) -> bool {
    crate::net::dp_objects::add_player_to_group(
        vars.dplay.get(),
        vars.send_to.get(),
        vars.player_id.get(),
    ) == 0
}
pub fn create_group(vars: &NetShared) -> Result<bool, Refused> {
    let made =
        crate::net::dp_objects::create_group(vars.dplay.get(), &crate::net::store::GROUP_NAME, &[]);
    if let Ok(id) = made {
        vars.send_to.set(id);
    }
    Ok(made.is_ok() && join(vars))
}
