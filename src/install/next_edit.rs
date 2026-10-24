//! `install::next_edit` (0x41530F): with ebx 0, the highlight (`g_listChoice`) moved on to
//! the first edit item (kind 4) from it, which begins to be edited (`install::start_edit`);
//! none before the list's end: the highlight back to 0. Any other ebx: nothing.
//!
//! In:  ebx 0 (a list action's start), edi the list
//! Out: every register kept (`pusha` / `popa`); the flags of what it did last (no caller
//!      reads them)
//! Callers `install::folder_page` (the folder page's action), `install::mark_page`.
//! Shadow mode compares. Self-test: lists of the test's against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{CHOICE, g, p, rb};

/// `state` the list action's (ebx): only the start (0) does anything.
pub fn next_edit(state: u32, list: u32) {
    if state != 0 {
        return;
    }
    let count = rb(list + 1) as u32;
    let item = |n: u32| list + 0x18 + 0x10 * n;
    // Converted: `rc2k_core::install::items::next_edit` (the highlight stored once, where
    // it ends).
    let (n, found) = rc2k_core::install::items::next_edit(g(CHOICE), count, |n| rb(item(n)));
    if found {
        p(CHOICE, n);
        crate::install::start_edit::start_edit(item(n));
    } else {
        p(CHOICE, 0);
    }
}
