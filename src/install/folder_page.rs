//! `install::folder_page` (0x4142F5): the install-folder page. At the start (ebx 0) the
//! first edit item is begun (`install::next_edit`). At the end (ebx negative) `dword_51BA00`
//! is 0. Then by `dword_51BA00`: not 1, the items at +0x38 and +0x48 get 0x40 (hidden);
//! not 2, +0x58 and +0x68 get 0x40.
//!
//! Takes the state, list.
//! A page action. Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments.

use crate::menu::list_handle::ListMenu;
use rc2k_core::install::pages;

/// `state` which call of the list action this is (0 the start, negative the end), `list` the page's.
pub fn folder_page(install: &'static crate::install::owner::Install, state: u32, list: ListMenu) {
    if state == 0 {
        crate::install::next_edit::next_edit(install, 0, list);
    }
    if (state as i32) < 0 {
        install.run.path_problem.set(0);
    }
    // Converted: `rc2k_core::install::pages::hidden_notices`.
    let hide = |offsets: [u32; 2]| {
        for o in offsets {
            let item = list.item_at_offset(o);
            item.set_first(item.first() | pages::HIDDEN);
        }
    };
    if pages::hidden_notices(install.run.path_problem.get()).0 {
        hide([0x38, 0x48]);
    }
    if pages::hidden_notices(install.run.path_problem.get()).1 {
        hide([0x58, 0x68]);
    }
}
