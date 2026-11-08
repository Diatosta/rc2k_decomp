//! `install::next_edit` (0x41530F): with ebx 0, the highlight (`g_listChoice`) moved on to
//! the first edit item (kind 4) from it, which begins to be edited (`install::start_edit`);
//! none before the list's end: the highlight back to 0. Any other ebx: nothing.
//!
//! Takes 0 (a list action's start), list.
//! Callers `install::folder_page` (the folder page's action), `install::mark_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments.

use crate::menu::list_handle::ListMenu;

/// `state` which call of the list action this is (0 the start, negative the end): only the start (0) does anything.
pub fn next_edit(install: &'static crate::install::owner::Install, state: u32, list: ListMenu) {
    if state != 0 {
        return;
    }
    let count = list.count() as u32;
    // Converted: `rc2k_core::install::items::next_edit` (the highlight stored once, where
    // it ends).
    let (n, found) = rc2k_core::install::items::next_edit(install.pages.choice.get(), count, |n| {
        list.item(n).first()
    });
    if found {
        install.pages.choice.set(n);
        crate::install::start_edit::start_edit(install, list.item(n));
    } else {
        install.pages.choice.set(0);
    }
}
