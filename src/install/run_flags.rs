//! What the program was asked to do at start, and the installer's run switches: the command
//! line's `-u`, whether an installation was just made, and the drive letter the drive list starts
//! at. Set from the command line by `system::read_command_line`, read by `system::main_loop` and
//! the installer.

use std::cell::Cell;

/// What the command line asked for, as `system::read_command_line` returns it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command {
    /// `-v`: the version is to be shown and the game left.
    pub show_version: bool,
    /// `-u`: the update run.
    pub updating: bool,
}

/// The run switches, on the game's thread.
pub struct RunFlags {
    /// The update run (`-u`).
    pub updating: Cell<bool>,
    /// An installation was just made (or is being opened).
    pub installing: Cell<bool>,
    /// The lack of room is ignored.
    pub ignore_room: Cell<bool>,
    /// The first drive letter the drive list looks at.
    pub first_drive: Cell<u8>,
    /// The string the main page's first item shows when the installer runs for an update.
    pub update_entry: Cell<Option<u16>>,
}

impl RunFlags {
    pub const fn new() -> Self {
        RunFlags {
            updating: Cell::new(false),
            installing: Cell::new(false),
            ignore_room: Cell::new(false),
            first_drive: Cell::new(b'c'),
            update_entry: Cell::new(None),
        }
    }

    /// Takes what the command line asked for.
    pub fn begin(&self, command: &Command) {
        if command.updating {
            self.updating.set(true);
        }
    }

    /// The command line's switches stop counting once the front end is reached.
    pub fn clear_command(&self) {
        self.installing.set(false);
        self.updating.set(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_takes_the_update_switch() {
        let flags = RunFlags::new();
        flags.begin(&Command {
            show_version: false,
            updating: true,
        });
        assert!(flags.updating.get() && !flags.installing.get());
        flags.clear_command();
        assert!(!flags.updating.get());
    }

    #[test]
    fn a_line_without_switches_changes_nothing() {
        let flags = RunFlags::new();
        flags.begin(&Command {
            show_version: false,
            updating: false,
        });
        assert!(!flags.updating.get() && !flags.installing.get() && !flags.ignore_room.get());
        assert_eq!(flags.first_drive.get(), b'c');
    }
}
