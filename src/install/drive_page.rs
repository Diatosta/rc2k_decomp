//! The drive page: at the start the highlight is set to the item of the typed drive letter; at
//! the end the typed letter becomes the highlighted drive's and `path_problem` is 0. Called by
//! `install::run_page`.

use crate::install::installer::rb;
use crate::list_handle::ListMenu;
use rc2k_core::install::pages;

/// `state`: which call of the list action this is (0 the start, negative the end).
pub fn drive_page(
    install: &'static crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    state: u32,
    list: ListMenu,
) {
    let letters = crate::place::BytePlace::from(&install.drive_rows.drive_rows);
    if state == 0 {
        let letter = crate::strings::same_text::lower(install.paths.typed.get(0)); // g_lowerCase
        let at = |n: u32| letters.add(pages::DRIVE_TEXT * n);
        let (_, highlight) = pages::drive_highlight(letter, |n| rb(at(n)), list.count());
        if let Some(highlight) = highlight {
            list_state.choice.set(highlight);
        }
    }
    if (state as i32) < 0
        && let Some(offset) = pages::chosen_drive(list_state.choice.get())
    {
        install.paths.typed.set(0, rb(letters.add(offset)));
        install.paths.typed.set(1, b':');
        install.run.path_problem.set(0);
    }
}
