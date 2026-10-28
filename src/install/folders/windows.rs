//! The original's folders (RAL.EXE's rules, kept as they were).

use super::{Current, Folders, Room};
use crate::errors::{BadInput, Refused};

pub struct Windows;

impl Folders for Windows {
    const DRIVES: bool = true;

    fn first_folder() -> Option<Vec<u8>> {
        None
    }

    fn normalised(path: &[u8], current: Current) -> Result<Option<Vec<u8>>, BadInput> {
        // Converted: `rc2k_core::install::paths::normalise`.
        rc2k_core::install::paths::normalise(
            path,
            &crate::strings::same_text::case_table(),
            current,
        )
    }

    fn bad_name(folder: &[u8]) -> Result<bool, BadInput> {
        // Converted: `rc2k_core::install::paths::bad_folder_name` (RAL.EXE's `unk_51DE18`).
        Ok(rc2k_core::install::paths::bad_folder_name(folder)?)
    }

    fn make(at: u32) -> Result<bool, Refused> {
        crate::install::make_folders::walk(at)
    }

    fn room(folder: &[u8]) -> Room {
        if folder.starts_with(b"\\\\") {
            return Room::Unmeasured;
        }
        let letter = folder.first().copied().unwrap_or(0) as u32;
        match crate::files::drive_free_space_kb::drive_free_space_kb(
            letter,
            &mut crate::install::installer::game_api,
        ) {
            Some(kb) => Room::Kb(kb),
            None => Room::Unreadable,
        }
    }

    fn shown(folder: &[u8]) -> Vec<u8> {
        folder.to_vec()
    }

    fn field_width(original: u16) -> u16 {
        original
    }
}
