//! DirectPlay's `EnumGroups` callback: the group whose short name is "susanivanovafanclub" is
//! the one to send to, and the enumeration stops; any other goes on. Returns 0 to stop, 1 to go
//! on.

use crate::errors::Refused;
use crate::net::link::Link;

/// The group's id, type and short name.
pub fn group_found(link: &Link, id: u32, kind: u32, short: &[u8]) -> Result<u32, Refused> {
    if kind != 0 {
        return Ok(1);
    }
    let order = crate::strings::same_text::compare_bytes(short, &crate::net::store::GROUP_NAME);
    if order != std::cmp::Ordering::Equal {
        return Ok(1);
    }
    link.set_send_to(id);
    Ok(0)
}

pub fn callback(link: &Link, id: u32, kind: u32, short: &[u8]) -> u32 {
    crate::net::lobby::callback_entry("net::group_found", || group_found(link, id, kind, short))
}
