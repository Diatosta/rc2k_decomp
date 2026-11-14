//! The installation-kind page: at the start the highlight is the media mode (2 when it is 0);
//! at the end the media mode becomes the chosen answer. Called by `install::run_page`.

use crate::install::installer::{media_mode, set_media_mode};

/// `state` which call of the list action this is (0 the start, negative the end).
pub fn mode_page(
    install: &crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    state: u32,
) {
    if state == 0 {
        list_state
            .choice
            .set(rc2k_core::install::pages::mode_highlight(media_mode()));
        install.run.path_problem.set(0);
    }
    if (state as i32) < 0 {
        set_media_mode(list_state.list_answer.get() as u32);
    }
}
