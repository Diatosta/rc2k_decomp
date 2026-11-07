//! `files::drive_type` (0x4031D8): what kind of drive a letter is.
//!
//! The letter is lowered through `g_lowerCase` and asked about as `x:\` (`sub_4031EE` makes that
//! text in a dword). Three callers: the search for the game's CD, the installer, and `sub_403310`,
//! the main loop's "is the base directory on a CD" check (whose answer this build no longer looks
//! at: nine `nop`s follow its call at 0x415448).

// ---- Rust version ----

/// The root that is asked about: `x:\`, or none for the current drive.
pub fn drive_root(letter: u8) -> Option<[u8; 4]> {
    // Converted: `rc2k_core::files::paths::drive_root`.
    rc2k_core::files::paths::drive_root(letter, crate::strings::same_text::lower)
}

pub fn drive_type(letter: u8) -> u32 {
    crate::os::drive_type(drive_root(letter))
}
