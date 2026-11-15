//! The network link: the DirectPlay objects this machine holds (the session object, the lobby
//! object, its player and the group messages go to), the session description it opens, and the
//! connection and session lists with the address being built. `Link` owns the objects' ids:
//! every release goes through it, and what is still held when it is dropped is released then.
//! The receiving thread reaches the ids through atomic cells, as before.

use crate::errors::Refused;
use crate::memory::owned::Owned;
use crate::memory::session::{Bytes, Dword};
use crate::net::dp_objects::Element;
use crate::net::store::{SessionDesc, zeros};
use crate::net::tables::NetTables;
use std::sync::Mutex;

/// The link's cells, shared with the receiving thread.
#[derive(rc2k_derive::Cells)]
pub struct LinkCells {
    /// The network's memory list (`memory::List`, 16 bytes) the tables are counted on.
    memory: [Dword; 4],
    /// The connections listed.
    pub connections: Dword,
    /// The sessions listed.
    pub sessions: Dword,
    /// 1 while a DirectPlay session is open.
    session_open: Dword,
    /// This player's DirectPlay id.
    player_id: Dword,
    /// The group the messages go to.
    send_to: Dword,
    /// The `IDirectPlay4A` object (0: none).
    dplay: Dword,
    /// The `IDirectPlayLobby2A` object (0: none).
    lobby: Dword,
    /// The machine's name, copied here by `net::host_session` before the session is opened on
    /// the receiving thread (`posted_open_host`), which may not reach the game thread's settings.
    host_name: Bytes<64>,
    /// The description of the session being opened or hosted.
    session_desc: SessionDesc,
}

pub struct Link {
    cells: LinkCells,
    tables: Owned<NetTables>,
    /// The address elements added since the last `clear_elements`.
    elements: Mutex<Vec<Element>>,
}

impl Link {
    pub const fn new() -> Self {
        Link {
            cells: LinkCells {
                memory: zeros(),
                connections: Dword::new(0),
                sessions: Dword::new(0),
                session_open: Dword::new(0),
                player_id: Dword::new(0),
                send_to: Dword::new(0),
                dplay: Dword::new(0),
                lobby: Dword::new(0),
                host_name: Bytes::new([0; 64]),
                session_desc: SessionDesc::zero(),
            },
            tables: Owned::new("no network lists"),
            elements: Mutex::new(Vec::new()),
        }
    }

    /// The cells, for the state capture of the tests.
    pub fn cells(&self) -> &LinkCells {
        &self.cells
    }

    // ---- the DirectPlay objects ----

    /// The DirectPlay object's id (0: none).
    pub fn dplay(&self) -> u32 {
        self.cells.dplay.get()
    }

    /// The DirectPlay object is made.
    pub fn has_dplay(&self) -> bool {
        self.dplay() != 0
    }

    /// A test's object kept as the DirectPlay object: the link releases it.
    #[cfg(test)]
    pub fn set_dplay(&self, id: u32) {
        self.cells.dplay.set(id);
    }

    /// The cell `co_create` puts the DirectPlay object's id in.
    pub fn dplay_cell(&self) -> &Dword {
        &self.cells.dplay
    }

    /// The lobby object's id (0: none).
    pub fn lobby(&self) -> u32 {
        self.cells.lobby.get()
    }

    /// The cell `co_create` puts the lobby object's id in.
    pub fn lobby_cell(&self) -> &Dword {
        &self.cells.lobby
    }

    /// This machine's player (0: none).
    pub fn player_id(&self) -> u32 {
        self.cells.player_id.get()
    }

    pub fn set_player_id(&self, id: u32) {
        self.cells.player_id.set(id);
    }

    /// The group the messages go to.
    pub fn send_to(&self) -> u32 {
        self.cells.send_to.get()
    }

    pub fn set_send_to(&self, id: u32) {
        self.cells.send_to.set(id);
    }

    /// The DirectPlay object released and forgotten: the references it has left, or none when
    /// there was no object.
    pub fn release_dplay(&self) -> Option<u32> {
        let id = self.dplay();
        if id == 0 {
            return None;
        }
        let left = crate::net::dp_objects::release(id);
        self.cells.dplay.set(0);
        Some(left)
    }

    /// The lobby object released and forgotten: the references it has left, or none when
    /// there was no object.
    pub fn release_lobby(&self) -> Option<u32> {
        let id = self.lobby();
        if id == 0 {
            return None;
        }
        let left = crate::net::dp_objects::release(id);
        self.cells.lobby.set(0);
        Some(left)
    }

    /// The session closed if one is open.
    pub fn close_session(&self) {
        if self.cells.session_open.get() == 1 {
            self.cells.session_open.set(0);
            crate::net::dp_objects::close(self.dplay());
        }
    }

    /// The session is open.
    pub fn mark_session_open(&self) {
        self.cells.session_open.set(1);
    }

    // ---- the session's description ----

    /// The session description.
    pub fn session_desc(&self) -> &SessionDesc {
        &self.cells.session_desc
    }

    /// The machine's name as a hosted session is opened with it.
    pub fn host_name(&self) -> &Bytes<64> {
        &self.cells.host_name
    }

    // ---- the lists ----

    /// The connections listed.
    pub fn connections(&self) -> &Dword {
        &self.cells.connections
    }

    /// The sessions listed.
    pub fn sessions(&self) -> &Dword {
        &self.cells.sessions
    }

    /// The network's memory list the tables are counted on.
    pub fn memory(&self) -> &[Dword; 4] {
        &self.cells.memory
    }

    /// The connection and session tables.
    pub fn tables(&self) -> &Owned<NetTables> {
        &self.tables
    }

    /// The tables taken (counted on the memory list) and both counts 0.
    pub fn take_tables(&self) -> Result<(), Refused> {
        crate::net::tables::take(self)?;
        self.cells.connections.set(0);
        self.cells.sessions.set(0);
        Ok(())
    }

    /// The tables given back and the counts 0. The memory list is freed by the caller.
    pub fn give_back_tables(&self) {
        self.tables.give_back();
        self.cells.connections.set(0);
        self.cells.sessions.set(0);
    }

    // ---- the address being built ----

    /// One more element of the compound address.
    pub fn push_element(&self, element: Element) {
        self.elements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(element);
    }

    /// No elements.
    pub fn clear_elements(&self) {
        self.elements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    /// The elements added.
    pub fn elements(&self) -> Vec<Element> {
        self.elements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl Drop for Link {
    /// What is still held is released: the DirectPlay object, then the lobby.
    fn drop(&mut self) {
        self.release_dplay();
        self.release_lobby();
    }
}
