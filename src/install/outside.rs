//! What the installation checks ask of the world outside the game's memory, as a trait.

use crate::errors::BadInput;
use crate::files::drive_type::drive_type;
use crate::install::normalise_path::{current_directory, normalised_bytes};
use crate::logfile::write_line::log_text;

pub trait Outside {
    fn file_exists(&mut self, path: &[u8]) -> Result<bool, BadInput>;
    fn drive_type(&mut self, letter: u8) -> u32;
    /// Makes `path` the current directory; whether it worked.
    fn set_directory(&mut self, path: &[u8]) -> bool;
    /// A line for the game's log.
    fn log(&mut self, text: &[u8]);
    /// Reads the installation's record (`install::record`) into the installer's names.
    fn read_installed(&mut self);
    // The same for the game and for a test:
    /// `install::normalise_path`: the path as written (with its NUL), None when not taken.
    fn normalise_path(&mut self, path: &[u8]) -> Result<Option<Vec<u8>>, BadInput> {
        normalised_bytes(path, &mut || current_directory())
    }
    /// The current directory with a `\` after it (and its NUL), if it could be had.
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

/// The outside world as the game has it: the installer's texts and the files.
pub struct Game {
    pub install: &'static crate::install::owner::Install,
}

impl Outside for Game {
    fn file_exists(&mut self, path: &[u8]) -> Result<bool, BadInput> {
        crate::files::file_exists::file_exists(path)
    }

    fn drive_type(&mut self, letter: u8) -> u32 {
        drive_type(letter)
    }

    fn set_directory(&mut self, path: &[u8]) -> bool {
        crate::files::directories::set_current_directory(path)
    }

    fn read_installed(&mut self) {
        crate::install::record::read_installed(self.install);
    }

    fn log(&mut self, text: &[u8]) {
        log_text(text, true, 1);
    }
}
