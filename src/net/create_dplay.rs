//! `net::create_dplay` (0x41861D): "NET: Create dplay", and DirectPlay's object made
//! (`CoCreateInstance` of CLSID_DirectPlay `stru_5AB330`, IID_IDirectPlay4A `stru_5AB310`,
//! in-process) into `g_dplay`.
//!
//! In:  nothing
//! Out: true when made (the original: carry clear, the flags of `test eax, eax`; every
//!      register kept), false for the carry
//! Caller: `net::find_connections`.

use crate::net::lobby::{DPLAY, a, co_create, log};

const TEXT: usize = 0x5C2E3A; // "NET: Create dplay"
pub const CLSID_DIRECTPLAY: usize = 0x5AB330; // stru_5AB330
pub const IID_DIRECTPLAY4A: usize = 0x5AB310; // stru_5AB310

/// The Rust version.
pub fn create_dplay() -> bool {
    log(a(TEXT), true, 2);
    co_create(a(CLSID_DIRECTPLAY), a(IID_DIRECTPLAY4A), a(DPLAY)) == 0
}
