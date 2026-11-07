//! `install::dxsetup_page` (0x41432F): at the page's end (ebx negative) with the answer
//! (`byte_51D415`) 1, `g_runDxSetup` is set so DirectX's set-up runs after the game.
//!
//! In:  ebx the state
//! Out: every register kept; the flags of its last test
//! Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

/// `state` the list action's (ebx).
pub fn dxsetup_page(install: &'static crate::install::owner::Install, state: u32) {
    // Converted: `rc2k_core::install::pages::runs_dxsetup`.
    if (state as i32) < 0
        && rc2k_core::install::pages::runs_dxsetup(install.pages.list_answer.get())
    {
        install.run.run_dxsetup.set(1);
    }
}
