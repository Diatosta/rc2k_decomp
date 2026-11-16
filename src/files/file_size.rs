//! The size of a file by its path: opened to read, its size taken, closed. Refuses what
//! `resolve_path` refuses.

use crate::errors::BadInput;

/// (the size, whether it worked).
pub fn file_size(path: &[u8]) -> Result<(u32, bool), BadInput> {
    Ok(rc2k_core::files::io::file_size(
        &mut crate::files::handle::GameFiles::new(),
        path,
    )?)
}
