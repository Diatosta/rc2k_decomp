//! The network's variables in one struct, reached through `net()` by code that has no
//! reference to it. The receiving thread reads and writes `vars` concurrently (its cells stay
//! atomic); the players' inboxes sit behind their own lock; `main` (this machine's settings, the
//! address being built, the session joined and each network screen's state) and the button
//! belong to the game thread.

use crate::net::link::Link;
use crate::net::store::{
    AddressVars, Button, Inboxes, JoinedVars, LobbyScreen, NetMain, NetShared, SessionsScreen,
    SettingsScreen, SetupVars,
};
use crate::session::GameThread;

/// The network's state.
pub struct Net {
    /// The network's variables the receiving thread shares (sessions, players, DirectPlay
    /// objects, chat).
    pub vars: NetShared,
    /// The DirectPlay objects, the session being opened and the lists of connections and
    /// sessions.
    pub link: Link,
    /// The players' inboxes of race messages, under the received messages' lock.
    pub inboxes: Inboxes,
    /// The variables the game's thread alone reaches.
    pub main: GameThread<NetMain>,
    /// The lobby button being drawn (the game thread's).
    pub button: GameThread<Button>,
    /// Whether a network game is on, this machine's car and the players' count: the race side
    /// and the drawing threads read it too.
    pub race_slot: rc2k_core::net::net_race::NetRaceSlot,
    /// Chat messages the receiving thread took, as (player, text with its NUL), until the game's
    /// thread adds them to the chat (`net::drain_chat`): measuring a line needs the fonts, which
    /// only the game's thread reaches.
    chat_waiting: std::sync::Mutex<Vec<(u32, Vec<u8>)>>,
}

impl Net {
    pub const fn new() -> Self {
        Net {
            vars: NetShared::new(),
            link: Link::new(),
            inboxes: Inboxes::new(),
            main: GameThread::new(NetMain::new()),
            button: GameThread::new(Button::new()),
            race_slot: rc2k_core::net::net_race::NetRaceSlot::new(),
            chat_waiting: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// A chat message from `player`, waiting for the game's thread.
    pub fn queue_chat(&self, player: u32, text: Vec<u8>) {
        self.chat_waiting
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push((player, text));
    }

    /// The waiting chat messages in the order they came, none left waiting.
    pub fn take_waiting_chat(&self) -> Vec<(u32, Vec<u8>)> {
        std::mem::take(
            &mut *self
                .chat_waiting
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    }

    /// Takes the received messages' lock: the players' inboxes and the chat's texts are read and
    /// written under it. It is not reentrant: the guard is passed down to what needs the inboxes.
    pub fn lock_received(&self) -> std::sync::MutexGuard<'_, [rc2k_core::net::inbox::Inbox; 8]> {
        self.inboxes.lock()
    }

    /// The network settings screen's state.
    pub fn settings(&self) -> &SettingsScreen {
        &self.main.get().settings
    }

    /// The sessions list screen's state.
    pub fn sessions(&self) -> &SessionsScreen {
        &self.main.get().sessions
    }

    /// This machine's network settings.
    pub fn setup(&self) -> &SetupVars {
        &self.main.get().setup
    }

    /// The address being built.
    pub fn address(&self) -> &AddressVars {
        &self.main.get().address
    }

    /// The lobby screen's state.
    pub fn lobby(&self) -> &LobbyScreen {
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
