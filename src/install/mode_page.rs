//! `install::mode_page` (0x41422B): the installation-kind page. At the start (ebx 0) the
//! highlight (`g_listChoice`) is `g_mediaMode` (or 2 when it is 0) and `dword_51BA00` is 0.
//! At the end (ebx negative) `g_mediaMode` becomes the chosen answer (`byte_51D415`).
//!
//! In:  ebx the state, edi the list
//! Out: eax as the original leaves it; the flags of `test ebx, ebx` last
//! Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{media_mode, set_media_mode};
use crate::install::vars::INSTALLER_PAGES;
use crate::install::vars::INSTALLER_RUN;

/// `state` the list action's (ebx).
pub fn mode_page(state: u32) {
    if state == 0 {
        // Converted: `rc2k_core::install::pages::mode_highlight`.
        INSTALLER_PAGES
            .choice
            .set(rc2k_core::install::pages::mode_highlight(media_mode()));
        INSTALLER_RUN.path_problem.set(0);
    }
    if (state as i32) < 0 {
        set_media_mode(INSTALLER_PAGES.list_answer.get() as u32);
    }
}
