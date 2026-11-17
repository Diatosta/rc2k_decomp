//! The Windows folder rules.

use super::{Current, Folders, Room};
use crate::errors::{BadInput, Refused};

pub struct Windows;

impl Folders for Windows {
    const DRIVES: bool = true;

    fn first_folder() -> Option<Vec<u8>> {
        None
    }

    fn normalised(path: &[u8], current: Current) -> Result<Option<Vec<u8>>, BadInput> {
        rc2k_core::install::paths::normalise(path, current)
    }

    fn bad_name(folder: &[u8]) -> Result<bool, BadInput> {
        // `folder` is the text without its NUL; zeros up to its third byte stand for the bytes
        // past a shorter text.
        let mut text = folder.to_vec();
        text.push(0);
        if text.len() < 3 {
            text.resize(3, 0);
        }
        Ok(rc2k_core::install::paths::bad_folder_name(text.as_slice())?)
    }

    fn make(at: crate::memory::place::BytePlace) -> Result<bool, Refused> {
        crate::install::make_folders::walk(at)
    }

    fn room(folder: &[u8], system: &crate::system::owner::System) -> Room {
        if folder.starts_with(b"\\\\") {
            return Room::Unmeasured;
        }
        let letter = folder.first().copied().unwrap_or(0) as u32;
        match crate::files::drive_free_space_kb::drive_free_space_kb(system, letter) {
            Some(kb) => Room::Kb(kb),
            None => Room::Unreadable,
        }
    }

    fn shown(folder: &[u8]) -> Vec<u8> {
        folder.to_vec()
    }

    fn field_most(record_most: u8) -> u8 {
        record_most
    }

    fn field_width(record_width: u16) -> u16 {
        record_width
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
