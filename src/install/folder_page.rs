//! The install-folder page: at the start the first edit item is begun (`install::next_edit`); at
//! the end `path_problem` is 0; then the items that do not match `path_problem` are hidden.
//! Called by `install::run_page`.

use crate::list_handle::ListMenu;
use rc2k_core::install::pages;

/// `state`: which call of the list action this is (0 the start, negative the end).
pub fn folder_page(
    install: &'static crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    state: u32,
    list: ListMenu,
) {
    if state == 0 {
        crate::install::next_edit::next_edit(install, list_state, 0, list);
    }
    if (state as i32) < 0 {
        install.run.path_problem.set(0);
    }
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
