//! `install::copy_counted` (0x4139DA): `files_copied` (the files copied so far) raised by one
//! and the file copied (`files::copy_file`, source to destination).
//!
//! Takes the source path, destination path.
//! Callers `install::install_files`, `install::install_clio`.

use crate::errors::Refused;
use crate::install::installer::copy_file;

/// `from` copied to `to`; whether it was all done (the original passes
/// `copy_file`'s carry on: the game's files' copies and the
/// Clio's stop on it). A failed copy is seen by the callers.
pub fn copy_counted(
    install: &crate::install::owner::Install,
    system: &crate::system::owner::System,
    from: impl Into<crate::place::BytePlace>,
    to: impl Into<crate::place::BytePlace>,
) -> Result<bool, Refused> {
    install
        .run
        .files_copied
        .set(install.run.files_copied.get().wrapping_add(1));
    copy_file(system, from.into(), to.into())
}
