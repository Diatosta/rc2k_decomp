//! The removal page: left with "keep the saves and settings" (answer 2), the uninstall script is
//! marked to end before those folders and the answer becomes the uninstall's (1). Called by
//! `install::run_page`.

/// `state` which call of the page's action this is (0 the start, negative the end).
pub fn removal_page(
    install: &crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    state: u32,
) {
    if (state as i32) < 0 && rc2k_core::install::pages::keeps_saves(list_state.list_answer.get()) {
        install
            .names
            .script
            .set(crate::install::vars::SCRIPT_MARK, 5);
        list_state.list_answer.set(1);
    }
}
