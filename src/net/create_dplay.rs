//! `net::create_dplay` (0x41861D): "NET: Create dplay", and DirectPlay's object made
//! (`CoCreateInstance` of CLSID_DirectPlay `stru_5AB330`, IID_IDirectPlay4A `stru_5AB310`,
//! in-process) into `g_dplay`.
//!
//! In:  nothing
//! Out: true when made (the original: carry clear, the flags of `test eax, eax`; every
//!      register kept), false for the carry
//! Caller: `net::find_connections`.

use crate::net::lobby::{co_create, log};
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

/// The Rust version.
pub fn create_dplay() -> bool {
    log(VIEWER_PANEL.text_ptr(texts::NET_CREATE_DPLAY), true, 2);
    co_create(
        &crate::net::owner::net().vars.clsid_dplay,
        &crate::net::owner::net().vars.iid_dplay4a,
        &crate::net::owner::net().vars.dplay,
    ) == 0
}
