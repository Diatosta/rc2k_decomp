//! What each page does at its start and at its end besides its items (the answers' way on is
//! `rc2k_core::install::flow`; the items and their marks are `rc2k_core::install::model`):
//! - language: at the end with a chosen answer the language is chosen and loaded;
//! - drive: at the start the highlight is on the item of the typed drive letter; at the end the
//!   typed letter becomes the highlighted drive's and `path_problem` is 0;
//! - folder: at the start the first edit item is begun; at the end `path_problem` is 0;
//! - no room: at the start the lack of room is no longer ignored;
//! - removal: left with "keep the saves and settings" (answer 2), the uninstall script is marked
//!   to end before those folders and the answer becomes the uninstall's (1).

use crate::errors::{BadInput, Refused};
use crate::install::installer::{load_language, refuse};
use crate::install::owner::Install;
use crate::install::vars::{PathProblem, fixed};
use rc2k_core::install::flow::Page;
use rc2k_core::install::model::PageDef;
use rc2k_core::install::pages;

/// The byte of the drives' texts at `offset` (the first byte of a text is its letter).
fn drive_letter(install: &Install, offset: u32) -> u8 {
    let rows = &install.drive_rows;
    let at = offset as usize;
    if at >= 400 {
        refuse(BadInput("a drive text past the drive rows"));
    }
    rows.get(at)
}

/// The page at its start: the highlight (`list_state.choice`) is moved, an edit begun.
pub(super) fn start(
    install: &Install,
    list_state: &crate::menu::list_state::ListState,
    page: Page,
    def: &mut PageDef,
) {
    match page {
        Page::Drive => {
            let letter = crate::strings::same_text::lower(
                install.paths.typed.text().first().copied().unwrap_or(0),
            );
            let (_, highlight) = pages::drive_highlight(
                letter,
                |n| drive_letter(install, pages::DRIVE_TEXT.wrapping_mul(n)),
                def.items.len() as u8,
            );
            if let Some(highlight) = highlight {
                list_state.write().choice = highlight;
            }
        }
        Page::Folder => {
            // The highlight moves on to the first edit item, which begins to be edited; none
            // before the list's end: the highlight back to 0.
            let (n, found) = def.next_edit(list_state.read().choice);
            if found {
                list_state.write().choice = n;
                def.start_edit(n);
            } else {
                list_state.write().choice = 0;
            }
        }
        Page::NoRoom => install.flags.ignore_room.set(false),
        Page::Language | Page::Main | Page::Removal => {}
    }
}

/// The page at its end.
pub(super) fn end(
    install: &'static Install,
    o: crate::owners::Owners<'static>,
    page: Page,
) -> Result<(), Refused> {
    let list_state = &o.menus.list;
    match page {
        Page::Language => {
            let answer = list_state.read().list_answer;
            if answer != 0 {
                crate::install::choose_language::choose_language(install, u32::from(answer))?;
                load_language(o)?;
            }
        }
        Page::Drive => {
            if let Some(offset) = pages::chosen_drive(list_state.read().choice) {
                install.paths.typed.set_drive(drive_letter(install, offset));
                install.run.path_problem.set(PathProblem::None);
            }
        }
        Page::Folder => install.run.path_problem.set(PathProblem::None),
        Page::Removal => {
            if pages::keeps_saves(list_state.read().list_answer) {
                install.names.script_mark.set(fixed::MARK_KEEP_SAVES);
                list_state.write().list_answer = 1;
            }
        }
        Page::Main | Page::NoRoom => {}
    }
    Ok(())
}
