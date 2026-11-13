//! Lists the DirectPlay connections ("NET: Find connections"): unless the receiving thread has
//! ended, makes the lobby object and DirectPlay's object and the lists' memory, then enumerates
//! the connections for the game's GUID. Returns true when some were listed. Games launched from a
//! lobby are not supported.

use crate::errors::Refused;
use crate::net::lobby::log_bytes;
use crate::net::store::NetShared;
use crate::net::{allocate_lists, create_dplay, open_lobby};
use crate::race::viewer_screen_store::{NetTexts, texts};
pub fn find_connections(
    vars: &NetShared,
    strings: &crate::strings::owner::Strings,
    lines: &NetTexts,
) -> Result<bool, Refused> {
    log_bytes(&lines.text(texts::NET_FIND_CONNECTIONS), true, 2);
    if vars.receive_done.get() == 1 || !open_lobby::open_lobby(vars, lines)? {
        return Ok(false);
    }
    if !create_dplay::create_dplay(vars, lines) {
        return Ok(false);
    }
    allocate_lists::allocate_lists(vars)?;
    vars.connections.set(0);
    let result = crate::net::dp_objects::enum_connections(
        &|guid, data, short, _flags, _context| {
            crate::net::connection_found::callback(vars, strings, lines, guid, data, short)
        },
        0,
    );
    // False when none was listed.
    Ok(result == 0 && vars.connections.get() >= 1)
}
