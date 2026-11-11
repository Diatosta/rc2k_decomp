//! `net::release_dplay` (0x41864E): with a DirectPlay object (`g_dplay` not 0), "crate::net::owner::NET_OWNER.vars:
//! Destroy dplay" logged (level 2, a line), the object released (`IDirectPlay4A` +8) and
//! `g_dplay` made 0.
//!
//! Caller: `net::destroy_connection`. `Release` from Rust (`net::dplay`).
//! Refuses nothing.

use crate::net::store::NetVars;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};
pub fn release_dplay(vars: &NetVars) {
    if vars.dplay.get() != 0 {
        crate::net::lobby::log_bytes(&VIEWER_PANEL.text(texts::NET_DESTROY_DPLAY), true, 2);
        crate::net::dp_objects::release(vars.dplay.get());
        vars.dplay.set(0);
    }
}
