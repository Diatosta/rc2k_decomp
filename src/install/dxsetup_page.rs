//! `install::dxsetup_page` (0x41432F): at the page's end (state negative) with the answer
//! (`list_answer`) 1, `g_runDxSetup` is set so DirectX's set-up runs after the game.
//!
//! Takes the state.
//! Caller `install::run_page`.

/// `state` which call of the list action this is (0 the start, negative the end).
pub fn dxsetup_page(install: &crate::install::owner::Install, state: u32) {
    // Converted: `rc2k_core::install::pages::runs_dxsetup`.
    if (state as i32) < 0
        && rc2k_core::install::pages::runs_dxsetup(install.pages.list_answer.get())
    {
        install.run.run_dxsetup.set(1);
    }
}
