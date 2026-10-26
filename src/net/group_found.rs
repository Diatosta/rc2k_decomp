//! `net::group_found` (0x419391): DirectPlay's `EnumGroups` callback: a group (type 0) whose
//! short name is "susanivanovafanclub" (`strings::same_text`) is the one to send to
//! (`g_netSendTo`), and the enumeration stops; any other: it goes on.
//!
//! In:  (stdcall) the group's id, its type, its name (a DPNAME: +8 the short name), the
//!      flags, the context
//! Out: eax = 0 found (stop), 1 go on; ebx, esi, edi, ebp kept; `ret 14h`
//! Caller: DirectPlay, from `net::join_group`. Its thunk is a `stdcall` Rust function
//! (`net::lobby::dispatch_callback`). Its call through `part`. Shadow mode runs the original
//! alone. Self-test: a copy called with test arguments, its call to a stand-in (the texts
//! compared), against the Rust version: the group, another name, another type.
//!
//! Hook removed 2026-10-23 (conversion phase 4): DirectPlay is given `callback`, the Rust
//! version; in the hooked DLL the game's own code passes the original. What the header
//! says of the thunk, shadow mode and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, ZERO_FLAG};
use crate::net::create_group::GROUP_NAME;
use crate::net::lobby::{Part, SAME_TEXT, SEND_TO, a, p};

/// The Rust version: eax.
pub fn group_found(args: &[u32], part: Part) -> Result<u32, Refused> {
    if args[1] != 0 {
        return Ok(1);
    }
    let mut r = Regs {
        eax: unsafe { ((args[2] + 8) as *const u32).read_unaligned() },
        ebx: a(GROUP_NAME),
        ..Regs::default()
    };
    part(SAME_TEXT, &mut r)?;
    if r.eflags & ZERO_FLAG == 0 {
        return Ok(1);
    }
    p(SEND_TO, args[0]);
    Ok(0)
}

/// The callback DirectPlay is given (no hook; 2026-10-23): the Rust version.
pub unsafe extern "system" fn callback(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
    crate::net::lobby::callback_entry("net::group_found", 0x419391, &[a, b, c, d, e], group_found)
}
