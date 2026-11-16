//! Opens a file for reading and closes it again. The last file path is none afterwards for a file
//! that exists and the resolved path for one that does not. Refuses what `resolve_path` refuses.

use crate::errors::BadInput;

/// `path` without its NUL.
pub fn file_exists(path: &[u8]) -> Result<bool, BadInput> {
    Ok(rc2k_core::files::io::file_exists(
        &mut crate::files::handle::GameFiles::new(),
        path,
    )?)
}
