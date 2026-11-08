//! Module net's owner (step 11, owned state): the network's variables in one struct,
//! reached through `net()` by the code that does not take it as a parameter. The receive
//! thread reads `vars` concurrently, as before.

use crate::net::store::{Button, NetVars};

/// The network's state.
pub struct Net {
    /// The network's variables (sessions, players, DirectPlay objects, chat).
    pub vars: NetVars,
    /// The lobby button being drawn.
    pub button: Button,
}

impl Net {
    pub const fn new() -> Self {
        Net {
            vars: NetVars::new(),
            button: Button::new(),
        }
    }
}

/// The one instance. Reached through `net()`; `moved` registers its stores' places.
pub static NET_OWNER: Net = Net::new();

/// The network's state, for a caller that does not take it as a parameter yet.
pub fn net() -> &'static Net {
    &NET_OWNER
}
