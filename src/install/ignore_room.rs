//! `install::ignore_room` (0x4142DA): the not-enough-room page's "go on anyway" action:
//! `dword_51BA04` 1, the answer (`byte_51D415`) 1 and the page left.
//!
//! In:  nothing
//! Out: every register kept; zero set, carry clear (`cmp eax, eax`)
//! An item action (kind 6). Caller `install::item_action` through a page's item.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{IGNORE_ROOM, LIST_ANSWER, LIST_LEAVE, p, pb};

pub fn ignore_room() {
    p(IGNORE_ROOM, 1);
    pb(LIST_ANSWER, 1);
    pb(LIST_LEAVE, 1);
}
