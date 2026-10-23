//! `net::connection_found` (0x4186E5): DirectPlay's `EnumConnections` callback: "NET: Found"
//! and the connection's name logged; with `g_noticeKind` 2, the two service providers
//! `unk_5AB370` and `unk_5AB380` (`system::same_guid`) left out; else the connection's data
//! copied into memory of the network's list (`memory::allocate`; none: the enumeration
//! stops) and, if fewer than 0x100 are listed, a record added (`net::connection_entry`):
//! +0x50 the data, +0x40 the service provider's GUID, +0 the name
//! (`strings::copy_printable`, 0x40 room).
//!
//! In:  (stdcall) the service provider's GUID, the connection, its size, its name (a
//!      DPNAME: +8 the short name), the flags, the context
//! Out: eax = 1 to go on, 0 to stop; ebx, esi, edi, ebp kept; `ret 18h`
//! A full list stops the enumeration after the memory is taken (it stays on the list).
//! Caller: DirectPlay, from `net::find_connections`. Its thunk is a `stdcall` Rust function
//! (`net::lobby::dispatch_callback`). Its calls through `part`. Shadow mode runs the original
//! alone. Self-test: a copy called with test arguments, its calls to stand-ins (the GUIDs
//! compared, the memory given, the records in a test table, the name copied), against the
//! Rust version: each way out.
//!
//! Hook removed 2026-10-23 (conversion phase 4): DirectPlay is given `callback`, the Rust
//! version; in the hooked DLL the game's own code passes the original. What the header
//! says of the thunk, shadow mode and the self-test was the hook's.

use crate::errors::Refused;
use crate::field;
use crate::framework::{Regs, ZERO_FLAG};
use crate::net::lobby::{
    ALLOCATE, CONNECTIONS, COPY_PRINTABLE, LOG_L2, LOG_L2_NL, NET_MEMORY, NOTICE_KIND, Part,
    SAME_GUID, a, g, gb, log_text, movsb, p,
};
use crate::net::types::NetConnection;
use rc2k_core::net::lists;

const FOUND: usize = 0x5C2E75; // "NET: Found"
const LEFT_OUT: [usize; 2] = [0x5AB370, 0x5AB380]; // unk_5AB370, unk_5AB380
const CONNECTION_ENTRY: u32 = 0x418530; // net::connection_entry

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The Rust version: eax.
pub fn connection_found(args: &[u32], part: Part) -> Result<u32, Refused> {
    let (guid, data, size, name) = (args[0], args[1], args[2], args[3]);
    let mut r = Regs::default();
    log_text(LOG_L2, a(FOUND), &r, part)?;
    let short = get(name + 8);
    log_text(LOG_L2_NL, short, &r, part)?;
    // Converted: `rc2k_core::net::lists` (`LEAVES_OUT_PROVIDERS`, `connection_listed`).
    if gb(NOTICE_KIND) == lists::LEAVES_OUT_PROVIDERS {
        for left_out in LEFT_OUT {
            r.eax = guid;
            r.ebx = a(left_out);
            part(SAME_GUID, &mut r)?;
            if r.eflags & ZERO_FLAG != 0 {
                return Ok(1);
            }
        }
    }
    r.eax = size;
    r.ebx = a(NET_MEMORY);
    part(ALLOCATE, &mut r)?;
    if r.carry() {
        return Ok(0);
    }
    let memory = r.eax;
    let Some(count) = lists::connection_listed(g(CONNECTIONS)) else {
        return Ok(0);
    };
    p(CONNECTIONS, count + 1);
    let mut e = Regs {
        eax: count,
        ebx: memory,
        ..r
    };
    part(CONNECTION_ENTRY, &mut e)?;
    let record = e.eax;
    unsafe { ((record + field!(NetConnection, data)) as *mut u32).write_unaligned(memory) };
    movsb(memory, data, size);
    movsb(record + field!(NetConnection, service_provider), guid, 0x10);
    let mut c = Regs {
        eax: short,
        ebx: record,
        ecx: 0x40,
        ..e
    };
    part(COPY_PRINTABLE, &mut c)?;
    Ok(1)
}

/// The callback DirectPlay is given (no hook; 2026-10-23): the Rust version.
pub unsafe extern "system" fn callback(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> u32 {
    crate::net::lobby::callback_entry(
        "net::connection_found",
        0x4186E5,
        &[a, b, c, d, e, f],
        connection_found,
    )
}
