//! The host's group, which every player joins and every message goes to, made under the name
//! "susanivanovafanclub" and the own player added to it. The name goes to `net::dp_objects` as
//! text. Returns true when made and joined.

use crate::errors::Refused;
use crate::net::link::Link;

/// The player put in the group (`AddPlayerToGroup`): whether it went.
pub fn join(link: &Link) -> bool {
    crate::net::dp_objects::add_player_to_group(link.dplay(), link.send_to(), link.player_id()) == 0
}
pub(super) fn create_group(link: &Link) -> Result<bool, Refused> {
    let made =
        crate::net::dp_objects::create_group(link.dplay(), &crate::net::store::GROUP_NAME, &[]);
    if let Ok(id) = made {
        link.set_send_to(id);
    }
    Ok(made.is_ok() && join(link))
}
