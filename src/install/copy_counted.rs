//! Raises `files_copied` by one and copies the file from the source path to the destination.
//! Called by `install::install_files`, `install::install_clio`.

use crate::errors::Refused;
use crate::install::installer::copy_file;

/// `from` copied to `to`; whether it was all done. The callers see a failed copy.
pub fn copy_counted(
    install: &crate::install::owner::Install,
    system: &crate::system::owner::System,
    from: impl Into<crate::memory::place::BytePlace>,
    to: impl Into<crate::memory::place::BytePlace>,
) -> Result<bool, Refused> {
    install
        .run
        .files_copied
        .set(install.run.files_copied.get().wrapping_add(1));
    copy_file(system, from.into(), to.into())
}
