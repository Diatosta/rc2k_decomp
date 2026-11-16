//! How the installer names, checks, makes and measures folders on each system, chosen at build
//! time as `Native` through `Folders`: `windows` (`c:\..\` lower case, its bad characters, the
//! drive's room, the drive page) and `unix` (native paths shown whole, `~` may be typed for the
//! home folder, no drive page, the room of the folder's file system). macOS takes the `unix`
//! rules with its own first folder. A new system is one more implementation and its line in
//! `Native`. Texts are Windows-1252 bytes, no NUL unless said.

use crate::errors::{BadInput, Refused};

#[cfg(unix)]
mod unix;
/// Whether an edit field's text is the folder typed on the page.
pub fn is_folder_field(
    install: &'static crate::install::owner::Install,
    text: crate::memory::place::BytePlace,
) -> bool {
    text.same_place(crate::memory::place::BytePlace::from(&install.paths.typed))
}

/// An edit field's most bytes: the folder field's by the system's rules, any other's its
/// record's (`record_most`).
pub fn field_most(
    install: &'static crate::install::owner::Install,
    text: crate::memory::place::BytePlace,
    record_most: u8,
) -> u8 {
    if is_folder_field(install, text) {
        <Native as Folders>::field_most(record_most)
    } else {
        record_most
    }
}

#[cfg(windows)]
mod windows;

/// The room where a folder would go.
pub enum Room {
    /// So many KB.
    Kb(u32),
    /// Not measured (a network path's).
    #[cfg_attr(not(windows), allow(dead_code))] // reason: only the Windows rules have one
    Unmeasured,
    /// It could not be read (the path is refused).
    Unreadable,
}

/// The current folder, asked for only when a path is empty.
pub type Current<'a> = &'a mut dyn FnMut() -> Result<Option<Vec<u8>>, BadInput>;

pub trait Folders {
    /// Whether a drive is chosen before the folder (the Windows drive page).
    const DRIVES: bool;

    /// The folder page's first text; None for the default (strings 0x9F, 0xA0).
    fn first_folder() -> Option<Vec<u8>>;

    /// A path (as typed or recorded) in the installer's one form of a folder: ending in its
    /// separator, with its NUL; None if the path is not taken. Empty: the current folder.
    fn normalised(path: &[u8], current: Current) -> Result<Option<Vec<u8>>, BadInput>;

    /// Whether a folder in that form has a name the system refuses.
    fn bad_name(folder: &[u8]) -> Result<bool, BadInput>;

    /// The folder at `at` (that form) and those above it made; whether it
    /// worked.
    fn make(at: crate::memory::place::BytePlace) -> Result<bool, Refused>;

    /// The room where the folder (that form) would go.
    fn room(folder: &[u8], system: &crate::system::owner::System) -> Room;

    /// A folder in that form as the folder page shows it.
    fn shown(folder: &[u8]) -> Vec<u8>;

    /// The folder field's width in pixels, given the field record's own width (its +6).
    fn field_width(record_width: u16) -> u16;

    /// The folder field's most bytes, given the field record's own most (its +1). The buffers hold
    /// 256, with the game's folders still to go after it.
    fn field_most(record_most: u8) -> u8;
}

/// An edit field's width: the folder field's by the system's rules, any other's its
/// record's (`record_width`). `text` is the field's text.
pub fn field_width(
    install: &'static crate::install::owner::Install,
    text: crate::memory::place::BytePlace,
    record_width: u16,
) -> u16 {
    if is_folder_field(install, text) {
        <Native as Folders>::field_width(record_width)
    } else {
        record_width
    }
}

#[cfg(windows)]
pub type Native = windows::Windows;
#[cfg(unix)]
pub type Native = unix::Unix;
