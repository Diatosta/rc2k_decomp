//! What kind of drive a letter is: the letter is lowered and asked about as `x:\`
//! (`files::drive_root`).

/// The root that is asked about: `x:\`, or none for the current drive.
pub fn drive_root(letter: u8) -> Option<[u8; 4]> {
    rc2k_core::files::paths::drive_root(letter, crate::strings::same_text::lower)
}

pub fn drive_type(letter: u8) -> u32 {
    crate::platform::os::drive_type(drive_root(letter))
}
