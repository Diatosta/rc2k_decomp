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

use crate::install::installer::{CHOICE, LIST_ANSWER, MEDIA_MODE, PATH_PROBLEM, g, gb, p};

/// `state` the list action's (ebx).
pub fn mode_page(state: u32) {
    if state == 0 {
        // Converted: `rc2k_core::install::pages::mode_highlight`.
        p(
            CHOICE,
            rc2k_core::install::pages::mode_highlight(g(MEDIA_MODE)),
        );
        p(PATH_PROBLEM, 0);
    }
    if (state as i32) < 0 {
        p(MEDIA_MODE, gb(LIST_ANSWER) as u32);
    }
}
