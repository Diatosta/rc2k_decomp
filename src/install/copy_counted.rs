//! Raises `files_copied` by one and copies the file from the source path to the destination.

use crate::errors::Refused;
use crate::install::installer::copy_file;

/// `from` copied to `to`; whether it was all done. The callers see a failed copy.
pub(super) fn copy_counted(
    install: &crate::install::owner::Install,
    from: impl Into<crate::memory::place::BytePlace>,
    to: impl Into<crate::memory::place::BytePlace>,
) -> Result<bool, Refused> {
    install
        .run
        .files_copied
        .set(install.run.files_copied.get().wrapping_add(1));
    copy_file(from.into(), to.into())
}
