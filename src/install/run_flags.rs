//! What the program was asked to do at start, and the installer's run switches: the command
//! line's `-u`, `-p` and `-i`, the drive letter the drive list starts at, and whether the
//! registration is offered. Set from the command line by `system::read_command_line`, read by
//! `system::main_loop` and the installer.

use std::cell::Cell;

/// The media mode and language digits a `-i` gave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GivenInstall {
    /// The media mode digit (above 0 selects it, held to 3).
    pub mode: i32,
    /// The language digit (above 0 selects it).
    pub language: i32,
}

/// What the command line asked for, as `system::read_command_line` returns it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command {
    /// `-v`: the version is to be shown and the game left.
    pub show_version: bool,
    /// `-u`: the update run.
    pub updating: bool,
    /// `-p`: installing.
    pub installing: bool,
    /// The last `-i`, if any: install from the paths given, with these digits.
    pub given: Option<GivenInstall>,
}

/// The run switches, on the game's thread.
pub struct RunFlags {
    /// The update run (`-u`).
    pub updating: Cell<bool>,
    /// Installing (`-p`, or an installation just made).
    pub installing: Cell<bool>,
    /// The paths were given on the command line (`-i`).
    pub from_command: Cell<bool>,
    /// The lack of room is ignored.
    pub ignore_room: Cell<bool>,
    /// `-i`'s media mode digit.
    pub given_mode: Cell<i32>,
    /// `-i`'s language digit.
    pub given_language: Cell<i32>,
    /// The first drive letter the drive list looks at.
    pub first_drive: Cell<u8>,
    /// The registration is offered.
    pub offer_registration: Cell<bool>,
}

impl RunFlags {
    pub const fn new() -> Self {
        RunFlags {
            updating: Cell::new(false),
            installing: Cell::new(false),
            from_command: Cell::new(false),
            ignore_room: Cell::new(false),
            given_mode: Cell::new(2),
            given_language: Cell::new(0),
            first_drive: Cell::new(b'c'),
            offer_registration: Cell::new(false),
        }
    }

    /// Takes what the command line asked for.
    pub fn begin(&self, command: &Command) {
        if command.updating {
            self.updating.set(true);
        }
        if command.installing {
            self.installing.set(true);
        }
        if let Some(given) = command.given {
            self.from_command.set(true);
            self.ignore_room.set(true);
            self.given_mode.set(given.mode);
            self.given_language.set(given.language);
        }
    }

    /// The command line's switches stop counting once the front end is reached.
    pub fn clear_command(&self) {
        self.installing.set(false);
        self.updating.set(false);
        self.from_command.set(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_takes_the_switches_and_the_given_digits() {
        let flags = RunFlags::new();
        flags.begin(&Command {
            show_version: false,
            updating: true,
            installing: false,
            given: Some(GivenInstall {
                mode: 1,
                language: 3,
            }),
        });
        assert!(flags.updating.get() && !flags.installing.get());
        assert!(flags.from_command.get() && flags.ignore_room.get());
        assert_eq!((flags.given_mode.get(), flags.given_language.get()), (1, 3));
        flags.clear_command();
        assert!(!flags.updating.get() && !flags.from_command.get());
        // The room is still ignored: only the command's switches are cleared.
        assert!(flags.ignore_room.get());
    }

    #[test]
    fn a_line_without_switches_changes_nothing() {
        let flags = RunFlags::new();
        flags.begin(&Command {
            show_version: false,
            updating: false,
            installing: false,
            given: None,
        });
        assert!(!flags.updating.get() && !flags.installing.get() && !flags.from_command.get());
        assert_eq!((flags.given_mode.get(), flags.first_drive.get()), (2, b'c'));
    }
}
