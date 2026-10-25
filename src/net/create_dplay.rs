//! `net::create_dplay` (0x41861D): "NET: Create dplay", and DirectPlay's object made
//! (`CoCreateInstance` of CLSID_DirectPlay `stru_5AB330`, IID_IDirectPlay4A `stru_5AB310`,
//! in-process) into `g_dplay`.
//!
//! In:  nothing
//! Out: true when made (the original: carry clear, the flags of `test eax, eax`; every
//!      register kept), false for the carry
//! Caller: `net::find_connections`.

use crate::net::lobby::{a, co_create, log};
use crate::net::store::NET;

const TEXT: usize = 0x5C2E3A; // "NET: Create dplay"

/// The Rust version.
pub fn create_dplay() -> bool {
    log(a(TEXT), true, 2);
    co_create(
        NET.clsid_dplay.addr() as u32,
        NET.iid_dplay4a.addr() as u32,
        NET.dplay.addr() as u32,
    ) == 0
}
