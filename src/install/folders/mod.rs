//! How the installer names, checks, makes and measures folders on each system (owner:
//! native on each, another system a plug-in). The installer's pages and steps
//! are the original's; every folder they handle passes through `Folders`, chosen at build
//! time as `Native`:
//! - `windows`: the original's own rules (`c:\..\` lower case, its bad characters, the
//!   walk into each folder, the drive's room, the drive page);
//! - `unix`: native paths (`/home/me/.local/share/rally-championship/`, shown whole, `~` may
//!   be typed for the home folder), no drive page, the room of the folder's file system.
//!
//! A new system (macOS: `~/Library/Application Support/..`) is one more implementation and
//! its line in `Native`. Texts are the game's (Windows-1252 bytes, no NUL unless said).

use crate::errors::{BadInput, Refused};

#[cfg(unix)]
mod unix;
/// Whether an edit field's text is the folder typed on the page.
pub fn is_folder_field(
    install: &'static crate::install::owner::Install,
    text: crate::place::BytePlace,
) -> bool {
    text.game_ptr() == crate::pointers::GamePtr::to(&install.paths.typed)
}

/// An edit field's most bytes: the folder field's by the system's rules, any other's its
/// record's (`original`).
pub fn field_most(text: crate::place::BytePlace, original: u8) -> u8 {
    let install = crate::install::owner::install();
    if is_folder_field(install, text) {
        <Native as Folders>::field_most(original)
    } else {
        original
    }
}

#[cfg(windows)]
mod windows;

/// The room where a folder would go.
pub enum Room {
    /// So many KB.
    Kb(u32),
    /// Not measured (the original skips a network path's).
    #[cfg_attr(not(windows), allow(dead_code))] // reason: only the Windows rules have one
    Unmeasured,
    /// It could not be read (the path is refused).
    Unreadable,
}

/// The current folder, asked for only when a path is empty.
pub type Current<'a> = &'a mut dyn FnMut() -> Result<Option<Vec<u8>>, BadInput>;

pub trait Folders {
    /// Whether a drive is chosen before the folder (the original's drive page).
    const DRIVES: bool;

    /// The folder page's first text; None for the original's (its strings 0x9F, 0xA0).
    fn first_folder() -> Option<Vec<u8>>;

    /// A path (as typed or recorded) in the installer's one form of a folder: ending in its
    /// separator, with its NUL; None if the path is not taken. Empty: the current folder.
    fn normalised(path: &[u8], current: Current) -> Result<Option<Vec<u8>>, BadInput>;

    /// Whether a folder in that form has a name the system refuses.
    fn bad_name(folder: &[u8]) -> Result<bool, BadInput>;

    /// The folder at `at` (that form) and those above it made; whether it
    /// worked.
    fn make(at: crate::place::BytePlace) -> Result<bool, Refused>;

    /// The room where the folder (that form) would go.
    fn room(folder: &[u8]) -> Room;

    /// A folder in that form as the folder page shows it.
    fn shown(folder: &[u8]) -> Vec<u8>;

    /// The folder field's width in pixels, given the original's (its record's +6).
    fn field_width(original: u16) -> u16;

    /// The folder field's most bytes, given the original's (its record's +1). The buffers
    /// hold 256, with the game's folders still to go after it.
    fn field_most(original: u8) -> u8;
}

/// An edit field's width: the folder field's by the system's rules, any other's its
/// record's (`original`). `text` is the field's text (its game address).
pub fn field_width(text: crate::place::BytePlace, original: u16) -> u16 {
    let install = crate::install::owner::install();
    if is_folder_field(install, text) {
        <Native as Folders>::field_width(original)
    } else {
        original
    }
}

#[cfg(windows)]
pub type Native = windows::Windows;
#[cfg(unix)]
pub type Native = unix::Unix;
