//! Module net's owner (step 11, owned state): the network's variables in one struct,
//! reached through `net()` by code that has no reference to it. The receive
//! thread reads and writes `vars` concurrently (its cells stay atomic); the button is the
//! game thread's.

use crate::net::store::{Button, NetVars};
use crate::session::GameThread;

/// The network's state.
pub struct Net {
    /// The network's variables (sessions, players, DirectPlay objects, chat).
    pub vars: NetVars,
    /// The lobby button being drawn (the game thread's).
    pub button: GameThread<Button>,
}

impl Net {
    pub const fn new() -> Self {
        Net {
            vars: NetVars::new(),
            button: GameThread::new(Button::new()),
        }
    }
}

/// The one instance. Reached through `net()`; `moved` registers its stores' places.
pub static NET_OWNER: Net = Net::new();

/// The network's state; pass it down where a function reaches it more than once.
pub fn net() -> &'static Net {
    &NET_OWNER
}
