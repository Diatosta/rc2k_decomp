//! DirectPlay's `Send` of `data` from this machine's player to `to`. Nothing checks for a
//! missing DirectPlay object, and no caller reads the result.

use crate::net::link::Link;
pub(super) fn dplay_send(link: &Link, to: u32, data: &[u8]) {
    crate::net::dp_objects::send(link.dplay(), link.player_id(), to, data);
}
