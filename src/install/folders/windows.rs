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
        // Converted: `rc2k_core::install::paths::bad_folder_name` (RAL.EXE's list of bad characters),
        // which reads to the NUL the original's place held: `folder` is the text without
        // it. Zeros up to its third byte stand for the place's bytes past a shorter text
        // (the original reads from there on whatever its length).
        let mut text = folder.to_vec();
        text.push(0);
        if text.len() < 3 {
            text.resize(3, 0);
        }
        Ok(rc2k_core::install::paths::bad_folder_name(text.as_slice())?)
    }

    fn make(at: crate::place::BytePlace) -> Result<bool, Refused> {
        crate::install::make_folders::walk(at)
    }

    fn room(folder: &[u8]) -> Room {
        if folder.starts_with(b"\\\\") {
            return Room::Unmeasured;
        }
        let letter = folder.first().copied().unwrap_or(0) as u32;
        match crate::files::drive_free_space_kb::drive_free_space_kb(
            crate::system::owner::system(),
            letter,
        ) {
            Some(kb) => Room::Kb(kb),
            None => Room::Unreadable,
        }
    }

    fn shown(folder: &[u8]) -> Vec<u8> {
        folder.to_vec()
    }

    fn field_most(original: u8) -> u8 {
        original
    }

    fn field_width(original: u16) -> u16 {
        original
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_names_are_read_without_their_nul() {
        assert_eq!(Windows::bad_name(b"c:\\games\\rally\\"), Ok(false));
        assert_eq!(Windows::bad_name(b"c:\\games\\\\rally\\"), Ok(true));
        assert_eq!(Windows::bad_name(b"c:\\ga?mes\\"), Ok(true));
        assert_eq!(Windows::bad_name(b"c"), Ok(false));
        assert_eq!(Windows::bad_name(b""), Ok(false));
    }
}
