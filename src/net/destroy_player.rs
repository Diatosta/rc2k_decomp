//! Calls DirectPlay's `DestroyPlayer` on the own player id and then clears the id, but only
//! when the id is 0 already (kept as is): the own player is never destroyed, `DestroyPlayer(0)`
//! is called instead.

use crate::net::link::Link;
pub(super) fn destroy_player(link: &Link) {
    let id = link.player_id();
    if id == 0 {
        crate::net::dp_objects::destroy_player(link.dplay(), id);
        link.set_player_id(0);
    }
}
