//! `net::release_dplay` (0x41864E): with a DirectPlay object (`g_dplay` not 0), "NET:
//! Destroy dplay" logged (level 2, a line), the object released (`IDirectPlay4A` +8) and
//! `g_dplay` made 0.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`; the flags are read by no caller:
//!      `lq -s 0x41864E`)
//! Caller: `net::destroy_connection`. `Release` from Rust (`net::dplay`, section 15).
//! Refuses nothing.

use crate::logfile::log_write::log_write;
use crate::net::dplay::{RELEASE, method};
use crate::net::lobby::{DPLAY, a, g, p};

const TEXT: usize = 0x5C2E4C; // "NET: Destroy dplay"

/// The Rust version.
pub fn release_dplay() {
    if g(DPLAY) != 0 {
        unsafe { log_write(a(TEXT) as *const u8, true, 2) };
        unsafe { method(g(DPLAY), RELEASE, &[]) };
        p(DPLAY, 0);
    }
}
