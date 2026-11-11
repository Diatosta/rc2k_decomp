//! `install::drive_page` (0x41425E): the drive page. At the start the highlight
//! (`g_listChoice`) is set to the item of the typed drive letter (`typed`, lowered
//! through `g_lowerCase`): its place in the drives' texts (`drive_rows`, 0x10 bytes each,
//! first byte the letter; 26 of them) plus 2, unless that is past the items. At
//! the end the typed letter becomes the highlighted drive's
//! (`g_listChoice - 2`, if not below 0, times 0x10 into the texts), the byte after it ':' and
//! `path_problem` 0.
//!
//! Takes the state, list.
//! Caller `install::run_page`.

use crate::install::installer::rb;
use crate::menu::list_handle::ListMenu;
use rc2k_core::install::pages;

/// `state` which call of the list action this is (0 the start, negative the end), `list` the page's.
pub fn drive_page(install: &'static crate::install::owner::Install, state: u32, list: ListMenu) {
    let letters = crate::place::BytePlace::from(&install.drive_rows.drive_rows);
    if state == 0 {
        let letter = crate::strings::same_text::lower(install.paths.typed.get(0)); // g_lowerCase
        // Converted: `rc2k_core::install::pages` (`drive_highlight`, `chosen_drive`).
        let at = |n: u32| letters.add(pages::DRIVE_TEXT * n);
        let (_, highlight) = pages::drive_highlight(letter, |n| rb(at(n)), list.count());
        if let Some(highlight) = highlight {
            install.pages.choice.set(highlight);
        }
    }
    if (state as i32) < 0
        && let Some(offset) = pages::chosen_drive(install.pages.choice.get())
    {
        install.paths.typed.set(0, rb(letters.add(offset)));
        install.paths.typed.set(1, b':');
        install.run.path_problem.set(0);
    }
}
