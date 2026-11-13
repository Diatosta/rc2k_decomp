//! DirectPlay's `EnumGroups` callback: the group whose short name is "susanivanovafanclub" is
//! the one to send to, and the enumeration stops; any other goes on. Returns 0 to stop, 1 to go
//! on.

use crate::errors::Refused;
use crate::net::store::NetShared;

/// The group's id, type and short name.
pub fn group_found(vars: &NetShared, id: u32, kind: u32, short: &[u8]) -> Result<u32, Refused> {
    if kind != 0 {
        return Ok(1);
    }
    let order = crate::strings::same_text::compare_bytes(short, &crate::net::store::GROUP_NAME);
    if order != std::cmp::Ordering::Equal {
        return Ok(1);
    }
    vars.send_to.set(id);
    Ok(0)
}

pub fn callback(vars: &NetShared, id: u32, kind: u32, short: &[u8]) -> u32 {
    crate::net::lobby::callback_entry("net::group_found", 0x419391, || {
        group_found(vars, id, kind, short)
    })
}
