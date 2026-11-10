//! `files::file_size` (0x402ED4): the size of a file by its path.
//!
//! Takes the path. Returns the size, or fails (0xFFFFFFFF in the original, unless only the close
//! failed). `open_by_mode` with mode 0, `files::handle_size` (0x402EA9, `GetFileSize`), `close`.
//! Two callers, the PNG reader and 0x4C6625. Refuses what `resolve_path` refuses. The name is a
//! reading.

use crate::errors::BadInput;

/// (eax, whether it worked).
pub fn file_size(path: &[u8]) -> Result<(u32, bool), BadInput> {
    Ok(rc2k_core::files::io::file_size(
        &mut crate::files::handle::GameFiles::new(),
        path,
    )?)
}
