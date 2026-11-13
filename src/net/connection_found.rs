//! DirectPlay's `EnumConnections` callback: logs the connection and, unless it is a serial or
//! modem provider (with the notice kind 2), copies its data into the network's memory and adds a
//! record to the list (up to 0x100). Returns 1 to go on, 0 to stop (out of memory or a full list).

use crate::errors::Refused;
use crate::net::lobby::log_bytes;
use crate::net::store::NetShared;
use crate::net::tables::{copy_printable_into, with_connection};
use crate::race::viewer_screen_store::{NetTexts, texts};
use crate::strings::owner::Strings;
use rc2k_core::net::lists;

/// The provider's GUID, the connection's address, its
/// short name.
pub fn connection_found(
    vars: &NetShared,
    strings: &Strings,
    lines: &NetTexts,
    guid: &[u8; 16],
    data: &[u8],
    short: &[u8],
) -> Result<u32, Refused> {
    log_bytes(&lines.text(texts::NET_FOUND), false, 2);
    log_bytes(short, true, 2);
    if crate::strings::load_language::notice_kind(strings) == lists::LEAVES_OUT_PROVIDERS
        && [&crate::net::store::SP_SERIAL, &crate::net::store::SP_MODEM]
            .into_iter()
            .any(|left_out| *left_out == *guid)
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

pub fn callback(
    vars: &NetShared,
    strings: &Strings,
    lines: &NetTexts,
    guid: &[u8; 16],
    data: &[u8],
    short: &[u8],
) -> u32 {
    crate::net::lobby::callback_entry("net::connection_found", 0x4186E5, || {
        connection_found(vars, strings, lines, guid, data, short)
    })
}
