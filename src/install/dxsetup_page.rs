//! At the page's end with the answer 1, sets the flag that runs DirectX's set-up after the
//! game. Called by `install::run_page`.

/// `state` which call of the list action this is (0 the start, negative the end).
pub fn dxsetup_page(
    install: &crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    state: u32,
) {
    if (state as i32) < 0 && rc2k_core::install::pages::runs_dxsetup(list_state.list_answer.get()) {
        install.run.run_dxsetup.set(1);
    }
}
