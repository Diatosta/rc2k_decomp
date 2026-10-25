//! `install::ignore_room` (0x4142DA): the not-enough-room page's "go on anyway" action:
//! `dword_51BA04` 1, the answer (`byte_51D415`) 1 and the page left.
//!
//! In:  nothing
//! Out: every register kept; zero set, carry clear (`cmp eax, eax`)
//! An item action (kind 6). Caller `install::item_action` through a page's item.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::vars::INSTALLER_PAGES;
use crate::install::vars::INSTALLER_RUN;

pub fn ignore_room() {
    INSTALLER_RUN.ignore_room.set(1);
    INSTALLER_PAGES.list_answer.set(1);
    INSTALLER_PAGES.list_leave.set(1);
}
