//! The network's variables in one struct, reached through `net()` by code that has no
//! reference to it. The receiving thread reads and writes `vars` concurrently (its cells stay
//! atomic); the players' inboxes sit behind their own lock; `main` (what is edited, the settings, the address, the session joined, the lobby's)
//! and the button belong to the game thread.

use crate::net::store::{
    AddressVars, Button, EditVars, Inboxes, JoinedVars, LobbyVars, NetMain, NetShared, SetupVars,
};
use crate::session::GameThread;

/// The network's state.
pub struct Net {
    /// The network's variables the receiving thread shares (sessions, players, DirectPlay
    /// objects, chat).
    pub vars: NetShared,
    /// The players' inboxes of race messages, under the received messages' lock.
    pub inboxes: Inboxes,
    /// The variables the game's thread alone reaches.
    pub main: GameThread<NetMain>,
    /// The lobby button being drawn (the game thread's).
    pub button: GameThread<Button>,
}

impl Net {
    pub const fn new() -> Self {
        Net {
            vars: NetShared::new(),
            inboxes: Inboxes::new(),
            main: GameThread::new(NetMain::new()),
            button: GameThread::new(Button::new()),
        }
    }

    /// Takes the received messages' lock: the players' inboxes and the chat's texts are read and
    /// written under it. It is not reentrant: the guard is passed down to what needs the inboxes.
    pub fn lock_received(&self) -> std::sync::MutexGuard<'_, [rc2k_core::net::inbox::Inbox; 8]> {
        self.inboxes.lock()
    }

    /// What is being edited and the chat being typed.
    pub fn edit(&self) -> &EditVars {
        &self.main.get().edit
    }

    /// This machine's network settings.
    pub fn setup(&self) -> &SetupVars {
        &self.main.get().setup
    }

    /// The address being built and the connection kept.
    pub fn address(&self) -> &AddressVars {
        &self.main.get().address
    }

    /// The lobby's variables.
    pub fn lobby(&self) -> &LobbyVars {
        &self.main.get().lobby
    }

    /// The session joined.
    pub fn joined(&self) -> &JoinedVars {
        &self.main.get().joined
    }
}

/// The one instance. Reached through `net()`; `moved` registers its stores' places.
pub static NET_OWNER: Net = Net::new();

/// The network's state; pass it down where a function reaches it more than once.
pub fn net() -> &'static Net {
    &NET_OWNER
}
