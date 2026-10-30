//! What the installation checks ask of the world outside the game's memory, as a trait.

use crate::errors::BadInput;
use crate::files::drive_type::drive_type;
use crate::install::normalise_path::{current_directory, normalised_bytes};
use crate::install::read_record::read_record;
use crate::logfile::write_line::log_text;

pub trait Outside {
    fn file_exists(&mut self, path: &[u8]) -> Result<bool, BadInput>;
    fn drive_type(&mut self, letter: u8) -> u32;
    /// `files::set_current_directory` (0x403105): whether it worked.
    fn set_directory(&mut self, path: &[u8]) -> bool;
    /// `install::read_record` on `var\ral.zog`.
    fn read_record(&mut self) -> Result<(), BadInput>;
    /// A line for the game's log.
    fn log(&mut self, text: &[u8]);
    /// `install::read_registry`.
    fn read_registry(&mut self) {
        crate::install::read_registry::read_registry_here();
    }
    // The same for the game and for a test:
    /// `install::normalise_path`: the path as written (with its NUL), None when not taken.
    fn normalise_path(&mut self, path: &[u8]) -> Result<Option<Vec<u8>>, BadInput> {
        normalised_bytes(path, &mut || current_directory())
    }
    /// `files::current_directory` (0x4030D1): the current directory with a `\` after it
    /// (and its NUL), if it could be had.
    fn current_directory(&mut self) -> Result<Option<Vec<u8>>, BadInput> {
        let Some(mut directory) = current_directory()? else {
            return Ok(None);
        };
        if !matches!(directory.last(), Some(b'\\' | b'/')) {
            directory.push(b'\\');
        }
        directory.push(0);
        Ok(Some(directory))
    }
}

pub struct Game;

impl Outside for Game {
    fn file_exists(&mut self, path: &[u8]) -> Result<bool, BadInput> {
        crate::files::file_exists::file_exists(path)
    }

    fn drive_type(&mut self, letter: u8) -> u32 {
        drive_type(letter)
    }

    fn set_directory(&mut self, path: &[u8]) -> bool {
        crate::files::directories::set_current_directory_to(path)
    }

    fn read_record(&mut self) -> Result<(), BadInput> {
        read_record(crate::pointers::GamePtr::to(
            &crate::install::installer::INSTALL_TEXTS.record_path,
        ))
    }

    fn log(&mut self, text: &[u8]) {
        log_text(text, true, 1);
    }
}
