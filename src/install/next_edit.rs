//! With state 0, the highlight moves on to the first edit item from it, which begins to be edited
//! (`install::start_edit`); none before the list's end: the highlight back to 0. Any other
//! state: nothing. Called by `install::folder_page`, `install::mark_page`.

use crate::list_handle::ListMenu;

/// `state`: which call of the list action this is (0 the start, negative the end); only 0 acts.
pub fn next_edit(
    install: &'static crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    state: u32,
    list: ListMenu,
) {
    if state != 0 {
        return;
    }
    let count = list.count() as u32;
    // The highlight is stored once, where it ends.
    let (n, found) = rc2k_core::install::items::next_edit(list_state.choice.get(), count, |n| {
        list.item(n).first()
    });
    if found {
        list_state.choice.set(n);
        crate::install::start_edit::start_edit(install, list.item(n));
    } else {
        list_state.choice.set(0);
    }
}
