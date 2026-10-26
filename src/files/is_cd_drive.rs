//! `files::is_cd_drive` (0x403310): whether a drive is a CD-ROM, in the flags.
//!
//! In:  eax low byte = the drive's letter, 0 for the current drive
//! Out: every register preserved (ecx, edx pushed, `pusha` .. `popa`); the flags of
//!      `cmp eax, 5` on `files::drive_type`'s answer: zero set for a CD-ROM, carry set
//!      below it
//! Two callers: the installer (0x413182) and `system::main_loop` (0x415443, the base
//! directory's drive; nine `nop`s after the call: the answer is not looked at). Manual
//! patch with a private copy (0xF bytes, the call relocated): the call starts at +3.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::files::drive_type::drive_type;

// ---- Rust version ----

/// (carry, zero): `cmp` of the drive's kind with a CD-ROM's.
pub fn is_cd_drive(letter: u8) -> (bool, bool) {
    // Converted: `rc2k_core::files::io::is_cd`.
    rc2k_core::files::io::is_cd(drive_type(letter))
}
