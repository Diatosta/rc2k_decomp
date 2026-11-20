//! What the installer's pages are made from and what they keep between visits: the facts
//! `rc2k_core::install::model::build` takes, the highlight each list keeps, and the folder text
//! shared with the rest of the installer (`InstallerPaths::typed`).

use crate::errors::Refused;
use crate::install::folders::{Folders, Native};
use crate::install::owner::Install;
use rc2k_core::install::model::{EditHost, Facts, List};
use std::cell::Cell;

/// The item each list had highlighted when it was left.
pub struct Highlights([Cell<u32>; 8]);

impl Highlights {
    pub const fn new() -> Self {
        let mut cells = [const { Cell::new(0) }; 8];
        let mut n = 0;
        while n < cells.len() {
            cells[n] = Cell::new(List::ALL[n].first_highlight());
            n += 1;
        }
        Self(cells)
    }

    pub fn get(&self, list: List) -> u32 {
        self.0[list as usize].get()
    }

    pub fn set(&self, list: List, highlight: u32) {
        self.0[list as usize].set(highlight);
    }
}

/// What the pages depend on, as the installer is now.
pub(super) fn facts(install: &Install) -> Facts {
    Facts {
        same_version: install.run.same_version.get(),
        installed: install.names.status.get() != 0,
        update_entry: install.flags.update_entry.get(),
        languages: install.names.language_count.get(),
        drives: install.run.drive_count.get(),
        path_problem: install.run.path_problem.get().code(),
        folder_most: <Native as Folders>::field_most(FOLDER_MOST),
        folder_width: <Native as Folders>::field_width(FOLDER_WIDTH),
    }
}

/// The folder field's most bytes and widest text before the system's rules.
const FOLDER_MOST: u8 = 0x7C;
const FOLDER_WIDTH: u16 = 0x2E9;

/// Where the folder field's text starts in the folder typed: past the drive's `c:` on Windows,
/// at its start where there are no drives.
pub(super) const TEXT_AT: usize = if <Native as Folders>::DRIVES { 2 } else { 0 };

/// The folder field's text: the folder typed from `TEXT_AT` to its NUL.
pub(super) fn typed_text(install: &Install) -> Vec<u8> {
    install.paths.typed.text_from(TEXT_AT)
}

/// `text` as the folder field's text (cut to its place, NUL-ended).
pub(super) fn store_typed_text(install: &Install, text: &[u8]) {
    install.paths.typed.set_from(TEXT_AT, text);
}

/// The font of an edit's kind.
pub(super) fn edit_font(
    install: &Install,
    font_kind: u8,
) -> Result<crate::gfx::typefaces::FontId, crate::errors::BadInput> {
    let font = crate::install::vars::InstallerPages::kind_font(install, 5 + 8 * font_kind as u32);
    crate::install::vars::InstallerPages::font_of(install, u32::from(font))
}

/// What the folder field's keys ask of the installer: the width of a text in the field's font,
/// the strings, the system's first folder.
pub(super) struct FolderHost {
    pub so: crate::strings::append_string_by_id::StringOwners<'static>,
    pub install: &'static Install,
    pub gfx: &'static crate::gfx::owner::Gfx,
    pub font_kind: u8,
    pub most: u8,
}

impl EditHost for FolderHost {
    type Error = Refused;

    fn text_width(&mut self, text: &[u8]) -> Result<u32, Refused> {
        let mut measured = text.to_vec();
        measured.push(0);
        // The measure takes the text's last place as its limit.
        let limit = u32::from(self.most)
            .wrapping_add(TEXT_AT as u32)
            .wrapping_add(1);
        let font = edit_font(self.install, self.font_kind)?;
        let (width, _) = crate::install::installer::measure_text(font, &measured, limit, self.gfx)?;
        Ok(width)
    }

    fn string(&mut self, id: u16) -> Vec<u8> {
        let mut text = Vec::new();
        crate::install::installer::push_string_by_id(self.so, &mut text, u32::from(id));
        text
    }

    fn first_folder(&mut self) -> Option<Vec<u8>> {
        // The folder typed holds it as the field's text, whole when the field starts at its start.
        <Native as Folders>::first_folder()
            .filter(|_| TEXT_AT == 0)
            .map(|mut first| {
                first.truncate(255);
                first
            })
    }
}
