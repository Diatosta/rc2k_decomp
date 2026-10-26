//! `install::copy_counted` (0x4139DA): `dword_51BAE8` (the files copied so far) raised by one
//! and the file copied (`files::copy_file`, eax the source esi, ebx the destination edi).
//!
//! In:  esi the source path, edi the destination path
//! Out: every register kept (`pusha` / `popa`)
//! Callers `install::install_files`, `install::install_clio`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::copy_file;
use crate::install::vars::INSTALLER_RUN;

/// `from` (esi) copied to `to` (edi); whether it was all done (the original leaves
/// `copy_file`'s carry through its `pusha` / `popa`: the game's files' copies and the
/// Clio's stop on it). Until 2026-10-23 the port dropped it, so a failed copy went unseen.
pub fn copy_counted(from: u32, to: u32) -> Result<bool, Refused> {
    INSTALLER_RUN
        .files_copied
        .set(INSTALLER_RUN.files_copied.get().wrapping_add(1));
    copy_file(from, to)
}
