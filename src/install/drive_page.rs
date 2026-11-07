//! `install::drive_page` (0x41425E): the drive page. At the start (ebx 0) the highlight
//! (`g_listChoice`) is set to the item of the typed drive letter (`byte_95EBF8`, lowered
//! through `g_lowerCase`): its place in the drives' texts (`byte_52A30F`, 0x10 bytes each,
//! first byte the letter; 26 of them) plus 2, unless that is past the items (`[edi+1]`). At
//! the end (ebx negative) the typed letter becomes the highlighted drive's
//! (`g_listChoice - 2`, if not below 0, times 0x10 into the texts), `byte_95EBF9` ':' and
//! `dword_51BA00` 0.
//!
//! In:  ebx the state, edi the list
//! Out: eax, ecx, edx as the original leaves them; esi kept; the flags of the end's `jb`
//! Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::rb;
use crate::menu::list_handle::ListMenu;
use rc2k_core::install::pages;

/// `state` the list action's (ebx), `list` the page's.
pub fn drive_page(install: &'static crate::install::owner::Install, state: u32, list: ListMenu) {
    let letters = crate::place::Place::from(&install.drive_rows.drive_rows);
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
