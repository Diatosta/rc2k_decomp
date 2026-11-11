//! The network's lists (`g_netConnectionTable`, `g_netSessionTable`, each connection's
//! data and the connection's compound address: blocks of the game's heap on `g_netMemory`),
//! Rust's: taken counted on the list as the game's blocks
//! were (`allocate_lists`, `connection_found`, `build_address`), given back with it
//! (`free_messages`). The counts stay in `crate::net::owner::net().vars.connections` / `crate::net::owner::net().vars.sessions`.

use crate::errors::{BadInput, Refused};
use crate::memory::owned::Owned;
use crate::net::lobby::fatal;
use crate::net::store::NetVars;
use crate::net::types::Guid;
use rc2k_core::net::lists::{
    MOST_LISTED, NO_CONNECTION, NO_CONNECTIONS, NO_SESSION, NO_SESSIONS, TABLES,
};

/// A connection found (`NetConnection`): its name, its service provider, its data.
#[derive(Clone)]
pub struct Connection {
    pub name: [u8; 0x40],
    pub provider: Guid,
    pub data: Vec<u8>,
}

/// A session found (`NetSession`).
#[derive(Clone, Copy)]
pub struct Session {
    pub name: [u8; 0x40],
    pub instance: Guid,
    pub max_players: u32,
    pub players: u32,
    pub user: [u8; 0x10],
}

pub struct NetTables {
    pub connections: Vec<Connection>,
    pub sessions: Vec<Session>,
    /// The compound address `build_address` makes (`g_netAddress`).
    pub address: Vec<u8>,
}

impl NetTables {
    fn new() -> Self {
        Self {
            connections: vec![
                Connection {
                    name: [0; 0x40],
                    provider: [0; 16],
                    data: Vec::new(),
                };
                MOST_LISTED as usize
            ],
            sessions: vec![
                Session {
                    name: [0; 0x40],
                    instance: [0; 16],
                    max_players: 0,
                    players: 0,
                    user: [0; 16],
                };
                MOST_LISTED as usize
            ],
            address: Vec::new(),
        }
    }
}

pub static NET_TABLES: Owned<NetTables> = Owned::new("no network lists");

/// The network's list (`g_netMemory`).
pub fn list(vars: &NetVars) -> &[crate::session::Dword; 4] {
    &vars.memory
}

/// `allocate_lists`: both tables taken, counted as the game's two blocks.
pub fn take(vars: &NetVars) -> Result<(), Refused> {
    NET_TABLES.take(list(vars), TABLES[0].0, NetTables::new())?;
    crate::memory::owned::count_on(list(vars), TABLES[1].0)
}

/// `bytes` more counted on the network's list (a block the original takes there).
pub fn count(vars: &NetVars, bytes: u32) -> Result<(), Refused> {
    crate::memory::owned::count_on(list(vars), bytes)
}

/// The original's `net::connection_entry` checks: a number not below the count, then no
/// table, are its fatal errors.
fn listed(index: u32, count: u32, codes: [u32; 2]) -> Result<(), Refused> {
    if index >= count {
        return Err(fatal(codes[0]).into());
    }
    if !NET_TABLES.is_taken() {
        return Err(fatal(codes[1]).into());
    }
    Ok(())
}

/// `f` on connection `index` (`net::connection_entry`'s checks first).
pub fn with_connection<R>(
    vars: &NetVars,
    index: u32,
    f: impl FnOnce(&mut Connection) -> R,
) -> Result<R, Refused> {
    listed(
        index,
        vars.connections.get(),
        [NO_CONNECTION, NO_CONNECTIONS],
    )?;
    NET_TABLES
        .with(|t| t.connections.get_mut(index as usize).map(f))?
        .ok_or(BadInput("a connection past the table").into())
}

/// `f` on session `index` (`net::session_entry`'s checks first).
pub fn with_session<R>(
    vars: &NetVars,
    index: u32,
    f: impl FnOnce(&mut Session) -> R,
) -> Result<R, Refused> {
    listed(index, vars.sessions.get(), [NO_SESSION, NO_SESSIONS])?;
    NET_TABLES
        .with(|t| t.sessions.get_mut(index as usize).map(f))?
        .ok_or(BadInput("a session past the table").into())
}

/// `strings::copy_printable` of the text `text` (none: empty) with `room` (its NUL
/// included) into `to`.
pub fn copy_printable_into(text: &[u8], to: &mut [u8], room: u32) -> Result<(), Refused> {
    let written = crate::strings::copy_printable::printable_from(text, room)?;
    to.get_mut(..written.len())
        .ok_or(BadInput("a name past its record"))?
        .copy_from_slice(&written);
    Ok(())
}

/// A record's name up to its NUL.
pub fn name_text(name: &[u8; 0x40]) -> Result<Vec<u8>, BadInput> {
    let end = name
        .iter()
        .position(|&b| b == 0)
        .ok_or(BadInput("a network name without its end"))?;
    Ok(name[..end].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::owner::net;

    #[test]
    fn a_connection_is_found_kept_and_found_again() {
        let net = net();
        let _lock = crate::store::test_lock();
        let figures = crate::memory::figures();
        let kept_list: Vec<u32> = net
            .vars
            .memory
            .iter()
            .map(crate::session::Dword::get)
            .collect();
        let bytes =
            |b: &crate::session::PlainBytes<16>| (0..16).map(|k| b.get(k)).collect::<Vec<u8>>();
        let kept_saved = bytes(&net.address().saved_connection);
        let kept_name: Vec<u8> = (0..64).map(|k| net.address().saved_name.get(k)).collect();
        let kept_count = net.vars.connections.get();
        net.vars.memory.iter().for_each(|d| d.set(0));
        take(&net.vars).unwrap();
        net.vars.connections.set(0);
        // DirectPlay's arguments: the provider's GUID, the data, the name record (+8 the
        // short name).
        let guid: [u8; 16] = std::array::from_fn(|k| k as u8 + 1);
        let data = vec![7u8; 5];
        let found =
            crate::net::connection_found::connection_found(&guid, &data, b"Test Link\0").unwrap();
        assert_eq!(found, 1);
        assert_eq!(net.vars.connections.get(), 1);
        let (name, provider, copied) =
            with_connection(&net.vars, 0, |c| (c.name, c.provider, c.data.clone())).unwrap();
        assert_eq!(provider[..], guid[..]);
        assert_eq!(copied, data);
        assert!(name_text(&name).unwrap().eq_ignore_ascii_case(b"test link"));
        assert!(with_connection(&net.vars, 1, |_| ()).is_err());
        crate::net::keep_connection::keep_connection(net, 0).unwrap();
        assert_eq!(
            crate::net::find_saved_connection::find_saved_connection(net).unwrap(),
            Some(0)
        );
        crate::net::free_messages::free_messages(&net.vars).unwrap();
        assert!(!NET_TABLES.is_taken());
        net.vars
            .memory
            .iter()
            .zip(kept_list)
            .for_each(|(d, v)| d.set(v));
        kept_saved
            .iter()
            .enumerate()
            .for_each(|(k, &v)| net.address().saved_connection.set(k, v));
        kept_name
            .iter()
            .enumerate()
            .for_each(|(k, &v)| net.address().saved_name.set(k, v));
        net.vars.connections.set(kept_count);
        crate::memory::with_figures(|f| *f = figures);
    }
}
