//! `net::create_dplay` (0x41861D): "NET: Create dplay", and DirectPlay's object made
//! (`CoCreateInstance` of CLSID_DirectPlay `clsid_dplay`, IID_IDirectPlay4A `iid_dplay4a`,
//! in-process) into `g_dplay`.
//!
//! Returns true when made.
//! Caller: `net::find_connections`.

use crate::net::lobby::{co_create, log_bytes};
use crate::net::store::NetVars;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};
pub fn create_dplay(vars: &NetVars) -> bool {
    log_bytes(&VIEWER_PANEL.text(texts::NET_CREATE_DPLAY), true, 2);
    co_create(&vars.clsid_dplay, &vars.iid_dplay4a, &vars.dplay) == 0
}
