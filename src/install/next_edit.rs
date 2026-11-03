//! `install::next_edit` (0x41530F): with ebx 0, the highlight (`g_listChoice`) moved on to
//! the first edit item (kind 4) from it, which begins to be edited (`install::start_edit`);
//! none before the list's end: the highlight back to 0. Any other ebx: nothing.
//!
//! In:  ebx 0 (a list action's start), edi the list
//! Out: every register kept (`pusha` / `popa`); the flags of what it did last (no caller
//!      reads them)
//! Callers `install::folder_page` (the folder page's action), `install::mark_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::vars::INSTALLER_PAGES;
use crate::menu::list_handle::ListMenu;

/// `state` the list action's (ebx): only the start (0) does anything.
pub fn next_edit(state: u32, list: ListMenu) {
    if state != 0 {
        return;
    }
    let count = list.count() as u32;
    // Converted: `rc2k_core::install::items::next_edit` (the highlight stored once, where
    // it ends).
    let (n, found) =
        rc2k_core::install::items::next_edit(INSTALLER_PAGES.choice.get(), count, |n| {
            list.item(n).first()
        });
    if found {
        INSTALLER_PAGES.choice.set(n);
        crate::install::start_edit::start_edit(list.item(n));
    } else {
        INSTALLER_PAGES.choice.set(0);
    }
}
