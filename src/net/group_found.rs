//! `net::group_found` (0x419391): DirectPlay's `EnumGroups` callback: a group (type 0) whose
//! short name is "susanivanovafanclub" (`strings::same_text`) is the one to send to
//! (`g_netSendTo`), and the enumeration stops; any other: it goes on.
//!
//! In:  (stdcall) the group's id, its type, its name (a DPNAME: +8 the short name), the
//!      flags, the context
//! Out: eax = 0 found (stop), 1 go on; ebx, esi, edi, ebp kept; `ret 14h`
//! Caller: DirectPlay, from `net::join_group` (`callback`, a `stdcall` Rust function).

use crate::errors::Refused;
use crate::net::store::NET;

/// The Rust version (see the header): the group's id, type and short name.
pub fn group_found(id: u32, kind: u32, short: &[u8]) -> Result<u32, Refused> {
    if kind != 0 {
        return Ok(1);
    }
    let order = crate::strings::same_text::compare_bytes(short, &NET.group_name.read());
    if order != std::cmp::Ordering::Equal {
        return Ok(1);
    }
    NET.send_to.set(id);
    Ok(0)
}

pub fn callback(id: u32, kind: u32, short: &[u8], _flags: u32, _context: u32) -> u32 {
    crate::net::lobby::callback_entry("net::group_found", 0x419391, || {
        group_found(id, kind, short)
    })
}
