//! `install::copy_counted` (0x4139DA): `dword_51BAE8` (the files copied so far) raised by one
//! and the file copied (`files::copy_file`, eax the source esi, ebx the destination edi).
//!
//! Takes the source path, destination path.
//! Callers `install::install_files`, `install::install_clio`.

use crate::errors::Refused;
use crate::install::installer::copy_file;

/// `from` copied to `to`; whether it was all done (the original passes
/// `copy_file`'s carry on: the game's files' copies and the
/// Clio's stop on it). Until 2026-10-23 the port dropped it, so a failed copy went unseen.
pub fn copy_counted(
    install: &'static crate::install::owner::Install,
    from: impl Into<crate::place::Place>,
    to: impl Into<crate::place::Place>,
) -> Result<bool, Refused> {
    let from: u32 = from.into().game();
    let to: u32 = to.into().game();
    install
        .run
        .files_copied
        .set(install.run.files_copied.get().wrapping_add(1));
    copy_file(from, to)
}
