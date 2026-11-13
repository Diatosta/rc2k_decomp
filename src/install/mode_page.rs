//! `install::mode_page` (0x41422B): the installation-kind page. At the start (state 0) the
//! highlight (`g_listChoice`) is `g_mediaMode` (or 2 when it is 0) and `path_problem` is 0.
//! At the end (state negative) `g_mediaMode` becomes the chosen answer (`list_answer`).
//!
//! Takes the state, list.
//! Caller `install::run_page`.

use crate::install::installer::{media_mode, set_media_mode};

/// `state` which call of the list action this is (0 the start, negative the end).
pub fn mode_page(install: &crate::install::owner::Install, state: u32) {
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
