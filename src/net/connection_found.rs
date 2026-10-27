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
//! Caller: DirectPlay, from `net::find_connections` (`callback`, a `stdcall` Rust function).

use crate::errors::Refused;
use crate::net::lobby::log;
use crate::net::store::NET;
use crate::net::tables::{copy_printable_into, with_connection};
use crate::pointers::GamePtr;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};
use crate::system::same_guid::same_guid;
use rc2k_core::net::lists;

/// A dword of the callback's argument (memory Rust owns: DirectPlay's name record).
fn get(at: u32) -> u32 {
    crate::pointers::read_or_end(at, "DirectPlay's name record")
}

/// The Rust version: eax.
pub fn connection_found(args: &[u32]) -> Result<u32, Refused> {
    let (guid, data, size, name) = (args[0], args[1], args[2], args[3]);
    log(VIEWER_PANEL.text_addr(texts::NET_FOUND), false, 2);
    let short = get(name + 8);
    log(short, true, 2);
    // Converted: `rc2k_core::net::lists` (`LEAVES_OUT_PROVIDERS`, `connection_listed`).
    if crate::strings::load_language::notice_kind() == lists::LEAVES_OUT_PROVIDERS
        && [&NET.sp_serial, &NET.sp_modem]
            .into_iter()
            .any(|left_out| same_guid(guid, left_out.addr()))
    {
        return Ok(1);
    }
    // The data's block (counted as the original's, taken also when not listed).
    crate::net::tables::count(size)?;
    let Some(count) = lists::connection_listed(NET.connections.get()) else {
        return Ok(0);
    };
    NET.connections.set(count + 1);
    let copied = GamePtr::new(data).read_bytes(size as usize, "a network callback's bytes")?;
    let provider = GamePtr::new(guid).read::<[u8; 16]>("a network callback's bytes")?;
    let mut name = with_connection(count, |c| c.name)?;
    copy_printable_into(short, &mut name, 0x40)?;
    with_connection(count, |c| {
        c.data = copied;
        c.provider = provider;
        c.name = name;
    })?;
    Ok(1)
}

/// The callback DirectPlay is given (`net::dp_objects`): the Rust version.
pub fn callback(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> u32 {
    crate::net::lobby::callback_entry(
        "net::connection_found",
        0x4186E5,
        &[a, b, c, d, e, f],
        connection_found,
    )
}
