//! `install::mode_page` (0x41422B): the installation-kind page. At the start (ebx 0) the
//! highlight (`g_listChoice`) is `g_mediaMode` (or 2 when it is 0) and `dword_51BA00` is 0.
//! At the end (ebx negative) `g_mediaMode` becomes the chosen answer (`byte_51D415`).
//!
//! Takes the state, list.
//! Caller `install::run_page`.

use crate::install::installer::{media_mode, set_media_mode};

/// `state` which call of the list action this is (0 the start, negative the end).
pub fn mode_page(install: &'static crate::install::owner::Install, state: u32) {
    if state == 0 {
        // Converted: `rc2k_core::install::pages::mode_highlight`.
        install
            .pages
            .choice
            .set(rc2k_core::install::pages::mode_highlight(media_mode()));
        install.run.path_problem.set(0);
    }
    if (state as i32) < 0 {
        set_media_mode(install.pages.list_answer.get() as u32);
    }
}
