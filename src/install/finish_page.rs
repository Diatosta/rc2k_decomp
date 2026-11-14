//! At the page's end with the answer 2, `script_mark` becomes 5 and the answer 1 (the readme is
//! shown). Called by `install::run_page`.

/// `state` which call of the list action this is (0 the start, negative the end).
pub fn finish_page(
    install: &crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    state: u32,
) {
    if (state as i32) < 0 && rc2k_core::install::pages::shows_readme(list_state.list_answer.get()) {
        install
            .names
            .script
            .set(crate::install::vars::SCRIPT_MARK, 5);
        list_state.list_answer.set(1);
    }
}
