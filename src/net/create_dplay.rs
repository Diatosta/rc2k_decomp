//! `net::create_dplay` (0x41861D): "NET: Create dplay", and DirectPlay's object made
//! (`CoCreateInstance` of CLSID_DirectPlay `clsid_dplay`, IID_IDirectPlay4A `iid_dplay4a`,
//! in-process) into `g_dplay`.
//!
//! Returns true when made.
//! Caller: `net::find_connections`.

use crate::net::lobby::{co_create, log_bytes};
use crate::net::store::NetShared;
use crate::race::viewer_screen_store::{NetTexts, texts};
pub fn create_dplay(vars: &NetShared, lines: &NetTexts) -> bool {
    log_bytes(&lines.text(texts::NET_CREATE_DPLAY), true, 2);
    co_create(
        &crate::net::store::CLSID_DPLAY,
        &crate::net::store::IID_DPLAY4A,
        &vars.dplay,
    ) == 0
}
