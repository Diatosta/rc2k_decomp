//! Calls DirectPlay's `DestroyPlayer` on the own player id and then clears the id, but only
//! when the id is 0 already (kept as is): the own player is never destroyed, `DestroyPlayer(0)`
//! is called instead.

use crate::net::store::NetShared;
pub fn destroy_player(vars: &NetShared) {
    let id = vars.player_id.get();
    if id == 0 {
        crate::net::dp_objects::destroy_player(vars.dplay.get(), id);
        vars.player_id.set(0);
    }
}
