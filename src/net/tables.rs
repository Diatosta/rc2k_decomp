//! The network's connection and session lists and the data hung on them, Rust's: taken counted
//! on the list (`allocate_lists`, `connection_found`, `build_address`), given back with it
//! (`free_messages`). The counts stay in `Link`'s cells.

use crate::errors::{BadInput, Refused};
use crate::net::link::Link;
use crate::net::lobby::fatal;
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
    /// The session is locked by a password.
    pub locked: bool,
}

pub struct NetTables {
    pub connections: Vec<Connection>,
    pub sessions: Vec<Session>,
    /// The compound address `build_address` makes.
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
                    locked: false,
                };
                MOST_LISTED as usize
            ],
            address: Vec::new(),
        }
    }
}

/// The network's list.
pub fn list(link: &Link) -> &[crate::memory::session::Dword; 4] {
    link.memory()
}

/// `allocate_lists`: both tables taken, counted as the game's two blocks.
pub fn take(link: &Link) -> Result<(), Refused> {
    link.tables()
        .take(list(link), TABLES[0].0, NetTables::new())?;
    crate::memory::owned::count_on(list(link), TABLES[1].0)
}

/// `bytes` more counted on the network's list.
pub fn count(link: &Link, bytes: u32) -> Result<(), Refused> {
    crate::memory::owned::count_on(list(link), bytes)
}

/// A number not below the count, then no table, are `net::connection_entry`'s fatal errors.
fn listed(link: &Link, index: u32, count: u32, codes: [u32; 2]) -> Result<(), Refused> {
    if index >= count {
        return Err(fatal(codes[0]).into());
    }
    if !link.tables().is_taken() {
        return Err(fatal(codes[1]).into());
    }
    Ok(())
}

/// `f` on connection `index` (`net::connection_entry`'s checks first).
pub(super) fn with_connection<R>(
    link: &Link,
    index: u32,
    f: impl FnOnce(&mut Connection) -> R,
) -> Result<R, Refused> {
    listed(
        link,
        index,
        link.connections().get(),
        [NO_CONNECTION, NO_CONNECTIONS],
    )?;
    link.tables()
        .with(|t| t.connections.get_mut(index as usize).map(f))?
        .ok_or(BadInput("a connection past the table").into())
}

/// `f` on session `index` (`net::session_entry`'s checks first).
pub(super) fn with_session<R>(
    link: &Link,
    index: u32,
    f: impl FnOnce(&mut Session) -> R,
) -> Result<R, Refused> {
    listed(
        link,
        index,
        link.sessions().get(),
        [NO_SESSION, NO_SESSIONS],
    )?;
    link.tables()
        .with(|t| t.sessions.get_mut(index as usize).map(f))?
        .ok_or(BadInput("a session past the table").into())
}

/// `strings::copy_printable` of the text `text` (none: empty) with `room` (its NUL
/// included) into `to`.
pub(super) fn copy_printable_into(text: &[u8], to: &mut [u8], room: u32) -> Result<(), Refused> {
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

    #[test]
    fn a_connection_is_found_kept_and_found_again() {
        let net = crate::test_support::own::net();
        let _lock = crate::test_support::test_lock();
        let figures = crate::memory::figures();
        let kept_list: Vec<u32> = net
            .link
            .memory()
            .iter()
            .map(crate::memory::session::Dword::get)
            .collect();
        let bytes = |b: &crate::memory::plain_buffer::PlainBuffer<16>| {
            (0..16).map(|k| b.get(k)).collect::<Vec<u8>>()
        };
        let kept_saved = bytes(&net.setup().saved_connection);
        let kept_name: Vec<u8> = (0..64).map(|k| net.settings().saved_name.get(k)).collect();
        let kept_count = net.link.connections().get();
        net.link.memory().iter().for_each(|d| d.set(0));
        take(&net.link).unwrap();
        net.link.connections().set(0);
        // DirectPlay's arguments: the provider's GUID, the data, the name record (+8 the
        // short name).
        let guid: [u8; 16] = std::array::from_fn(|k| k as u8 + 1);
        let data = vec![7u8; 5];
        let found = crate::net::connection_found::connection_found(
            &net.link,
            crate::test_support::own::strings(),
            &crate::race::viewer_screen_store::NET_TEXTS,
            &guid,
            &data,
            b"Test Link\0",
        )
        .unwrap();
        assert_eq!(found, 1);
        assert_eq!(net.link.connections().get(), 1);
        let (name, provider, copied) =
            with_connection(&net.link, 0, |c| (c.name, c.provider, c.data.clone())).unwrap();
        assert_eq!(provider[..], guid[..]);
        assert_eq!(copied, data);
        assert!(name_text(&name).unwrap().eq_ignore_ascii_case(b"test link"));
        assert!(with_connection(&net.link, 1, |_| ()).is_err());
        crate::net::keep_connection::keep_connection(net, 0).unwrap();
        assert_eq!(
            crate::net::find_saved_connection::find_saved_connection(net).unwrap(),
            Some(0)
        );
        crate::net::free_messages::free_messages(net).unwrap();
        assert!(!net.link.tables().is_taken());
        net.link
            .memory()
            .iter()
            .zip(kept_list)
            .for_each(|(d, v)| d.set(v));
        kept_saved
            .iter()
            .enumerate()
            .for_each(|(k, &v)| net.setup().saved_connection.set(k, v));
        kept_name
            .iter()
            .enumerate()
            .for_each(|(k, &v)| net.settings().saved_name.set(k, v));
        net.link.connections().set(kept_count);
        crate::memory::with_figures(|f| *f = figures);
    }
}
