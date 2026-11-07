//! `files::file_exists` (0x402EBE): opens a file for reading and closes it again.
//!
//! Takes path. `open_by_mode` with mode 0, then `close`. `g_lastFilePath` is 0 after a file that
//! exists and the resolved path after one that does not. 20 callers. Refuses what `resolve_path`
//! refuses. The name is a reading.

use crate::errors::BadInput;

// ---- Rust version ----

/// `path` without its NUL.
pub fn file_exists(path: &[u8]) -> Result<bool, BadInput> {
    Ok(rc2k_core::files::io::file_exists(
        &mut crate::files::handle::GameFiles::new(),
        path,
    )?)
}
