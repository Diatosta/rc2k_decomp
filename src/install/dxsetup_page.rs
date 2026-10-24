//! `install::dxsetup_page` (0x41432F): at the page's end (ebx negative) with the answer
//! (`byte_51D415`) 1, `g_runDxSetup` is set so DirectX's set-up runs after the game.
//!
//! In:  ebx the state
//! Out: every register kept; the flags of its last test
//! Caller `install::run_page`. Shadow mode compares.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{LIST_ANSWER, RUN_DXSETUP, gb, p};

/// `state` the list action's (ebx).
pub fn dxsetup_page(state: u32) {
    // Converted: `rc2k_core::install::pages::runs_dxsetup`.
    if (state as i32) < 0 && rc2k_core::install::pages::runs_dxsetup(gb(LIST_ANSWER)) {
        p(RUN_DXSETUP, 1);
    }
}
