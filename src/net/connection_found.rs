//! `net::connection_found` (0x4186E5): DirectPlay's `EnumConnections` callback: "NET: Found"
//! and the connection's name logged; with `g_noticeKind` 2, the two service providers
//! `sp_serial` and `sp_modem` (`system::same_guid`) left out; else the connection's data
//! copied into memory of the network's list (`memory::allocate`; none: the enumeration
//! stops) and, if fewer than 0x100 are listed, a record added (`net::connection_entry`):
//! +0x50 the data, +0x40 the service provider's GUID, +0 the name
//! (`strings::copy_printable`, 0x40 room).
//!
//! Takes (stdcall) the service provider's GUID, the connection, its size, its name (a DPNAME:
//! +8 the short name), the flags and the context. Returns 1 to go on, 0 to stop.
//! A full list stops the enumeration after the memory is taken (it stays on the list).
//! Caller: DirectPlay, from `net::find_connections` (`callback`, a `stdcall` Rust function).

use crate::errors::Refused;
use crate::net::lobby::log_bytes;
use crate::net::owner::net;
use crate::net::tables::{copy_printable_into, with_connection};
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};
use rc2k_core::net::lists;

/// The provider's GUID, the connection's address, its
/// short name.
pub fn connection_found(guid: &[u8; 16], data: &[u8], short: &[u8]) -> Result<u32, Refused> {
    let vars = &net().vars;
    log_bytes(&VIEWER_PANEL.text(texts::NET_FOUND), false, 2);
    log_bytes(short, true, 2);
    if crate::strings::load_language::notice_kind() == lists::LEAVES_OUT_PROVIDERS
        && [&vars.sp_serial, &vars.sp_modem]
            .into_iter()
            .any(|left_out| left_out.read() == *guid)
    {
        return Ok(1);
    }
    crate::net::tables::count(vars, data.len() as u32)?;
    let Some(count) = lists::connection_listed(vars.connections.get()) else {
        return Ok(0);
    };
    vars.connections.set(count + 1);
    let mut name = with_connection(vars, count, |c| c.name)?;
    copy_printable_into(short, &mut name, 0x40)?;
    with_connection(vars, count, |c| {
        c.data = data.to_vec();
        c.provider = *guid;
        c.name = name;
    })?;
    Ok(1)
}

pub fn callback(guid: &[u8; 16], data: &[u8], short: &[u8], _flags: u32, _context: u32) -> u32 {
    crate::net::lobby::callback_entry("net::connection_found", 0x4186E5, || {
        connection_found(guid, data, short)
    })
}
