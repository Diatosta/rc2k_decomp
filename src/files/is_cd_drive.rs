//! Whether a drive is a CD-ROM, in the flags; the letter 0 is the current drive.

use crate::files::drive_type::drive_type;

/// (carry, zero): `cmp` of the drive's kind with a CD-ROM's.
pub fn is_cd_drive(letter: u8) -> (bool, bool) {
    rc2k_core::files::io::is_cd(drive_type(letter))
}
