//! `install::room_page` (0x4142C7): the not-enough-room page. At the start (ebx 0)
//! `dword_51BA04` (ignore the room) is set to 0.
//!
//! In:  ebx the state
//! Out: every register kept; the flags of `test ebx, ebx` last
//! Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{IGNORE_ROOM, p};

/// `state` the list action's (ebx).
pub fn room_page(state: u32) {
    if state == 0 {
        p(IGNORE_ROOM, 0);
    }
}
