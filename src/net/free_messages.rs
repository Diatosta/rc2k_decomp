//! `net::free_messages` (0x4185BB): the network's memory given back (`memory::free_list`
//! on `stru_52A928`) and four of its counts (`dword_52AC34`, `dword_52AC38`,
//! `dword_52ACF4`, `dword_52ACF8`) made 0.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`)
//!
//! See `net/mod.rs`. One caller, `net::destroy_connection`. Refuses what
//! `memory::free_list` refuses.

use crate::errors::Refused;
use crate::net::{a, p};

const LIST: usize = 0x52A928; // stru_52A928
const COUNTS: [usize; 4] = [0x52AC34, 0x52AC38, 0x52ACF4, 0x52ACF8];

/// The Rust version.
pub fn free_messages() -> Result<(), Refused> {
    unsafe { crate::memory::free_list::free_list(a(LIST) as *mut _) }?;
    for at in COUNTS {
        p(at, 0);
    }
    Ok(())
}
