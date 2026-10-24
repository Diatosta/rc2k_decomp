//! `net::group_found` (0x419391): DirectPlay's `EnumGroups` callback: a group (type 0) whose
//! short name is "susanivanovafanclub" (`strings::same_text`) is the one to send to
//! (`g_netSendTo`), and the enumeration stops; any other: it goes on.
//!
//! In:  (stdcall) the group's id, its type, its name (a DPNAME: +8 the short name), the
//!      flags, the context
//! Out: eax = 0 found (stop), 1 go on; ebx, esi, edi, ebp kept; `ret 14h`
//! Caller: DirectPlay, from `net::join_group` (`callback`, a `stdcall` Rust function).

use crate::errors::Refused;
use crate::net::create_group::GROUP_NAME;
use crate::net::lobby::{SEND_TO, a, p};

/// The Rust version: eax.
pub fn group_found(args: &[u32]) -> Result<u32, Refused> {
    if args[1] != 0 {
        return Ok(1);
    }
    let name = unsafe { ((args[2] + 8) as *const u32).read_unaligned() };
    let order = unsafe {
        crate::strings::same_text::compare_texts(name as *const u8, a(GROUP_NAME) as *const u8)
    };
    if order != std::cmp::Ordering::Equal {
        return Ok(1);
    }
    p(SEND_TO, args[0]);
    Ok(0)
}

/// The callback DirectPlay is given: the Rust version.
pub unsafe extern "system" fn callback(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
    crate::net::lobby::callback_entry("net::group_found", 0x419391, &[a, b, c, d, e], group_found)
}
