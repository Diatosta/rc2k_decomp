//! Raises `files_copied` by one and copies the file from the source path to the destination.

use crate::errors::Refused;
use crate::install::installer::copy_file;

pub(super) fn copy_counted(
    install: &crate::install::owner::Install,
    from: &[u8],
    to: &[u8],
) -> Result<bool, Refused> {
    install
        .run
        .files_copied
        .set(install.run.files_copied.get().wrapping_add(1));
    copy_file(from, to)
}
