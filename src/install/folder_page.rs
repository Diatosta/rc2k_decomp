//! `install::folder_page` (0x4142F5): the install-folder page. At the start (ebx 0) the
//! first edit item is begun (`install::next_edit`). At the end (ebx negative) `dword_51BA00`
//! is 0. Then by `dword_51BA00`: not 1, the items at +0x38 and +0x48 get 0x40 (hidden);
//! not 2, +0x58 and +0x68 get 0x40.
//!
//! In:  ebx the state, edi the list
//! Out: every register kept but what `next_edit` leaves; the flags of the last `cmp`
//! A page action. Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::menu::list_handle::ListMenu;
use rc2k_core::install::pages;

/// `state` the list action's (ebx), `list` the page's.
pub fn folder_page(g: &'static crate::install::owner::Install, state: u32, list: ListMenu) {
    if state == 0 {
        crate::install::next_edit::next_edit(g, 0, list);
    }
    if (state as i32) < 0 {
        g.run.path_problem.set(0);
    }
    // Converted: `rc2k_core::install::pages::hidden_notices`.
    let hide = |offsets: [u32; 2]| {
        for o in offsets {
            let item = list.item_at_offset(o);
            item.set_first(item.first() | pages::HIDDEN);
        }
    };
    if pages::hidden_notices(g.run.path_problem.get()).0 {
        hide([0x38, 0x48]);
    }
    if pages::hidden_notices(g.run.path_problem.get()).1 {
        hide([0x58, 0x68]);
    }
}
