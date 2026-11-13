//! DirectPlay's `Send` of `data` from this machine's player to `to`. Nothing checks for a
//! missing DirectPlay object, and no caller reads the result.

use crate::net::store::NetShared;
pub fn dplay_send(vars: &NetShared, to: u32, data: &[u8]) {
    crate::net::dp_objects::send(vars.dplay.get(), vars.player_id.get(), to, data);
}
