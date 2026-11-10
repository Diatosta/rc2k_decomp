//! `files::is_cd_drive` (0x403310): whether a drive is a CD-ROM, in the flags.
//!
//! Takes low byte = the drive's letter, 0 for the current drive. Two callers: the installer
//! (0x413182) and `system::main_loop` (0x415443, the base directory's drive; nine `nop`s after the
//! call: the answer is not looked at).

use crate::files::drive_type::drive_type;

/// (carry, zero): `cmp` of the drive's kind with a CD-ROM's.
pub fn is_cd_drive(letter: u8) -> (bool, bool) {
    // Converted: `rc2k_core::files::io::is_cd`.
    rc2k_core::files::io::is_cd(drive_type(letter))
}
