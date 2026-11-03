//! `net::release_dplay` (0x41864E): with a DirectPlay object (`g_dplay` not 0), "NET:
//! Destroy dplay" logged (level 2, a line), the object released (`IDirectPlay4A` +8) and
//! `g_dplay` made 0.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`; the flags are read by no caller:
//!      `lq -s 0x41864E`)
//! Caller: `net::destroy_connection`. `Release` from Rust (`net::dplay`).
//! Refuses nothing.

use crate::net::store::NET;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

/// The Rust version.
pub fn release_dplay() {
    if NET.dplay.get() != 0 {
        crate::logfile::log_write::log_at(VIEWER_PANEL.text_ptr(texts::NET_DESTROY_DPLAY), true, 2);
        crate::net::dp_objects::release(NET.dplay.get());
        NET.dplay.set(0);
    }
}
