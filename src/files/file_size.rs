//! `files::file_size` (0x402ED4): the size of a file by its path.
//!
//! In:  eax = path
//! Out: eax = the size, carry clear; or carry set (eax is then 0xFFFFFFFF, unless only the
//!      close failed); everything else preserved
//! `open_by_mode` with mode 0, `files::handle_size` (0x402EA9, `GetFileSize`), `close`. Two
//! callers, the PNG reader and `sub_4C6625`.
//! Refuses what `resolve_path` refuses. The name is a reading.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;

// ---- Rust version ----

/// (eax, whether it worked).
pub fn file_size(path: &[u8]) -> Result<(u32, bool), BadInput> {
    Ok(rc2k_core::files::io::file_size(
        &mut crate::files::handle::GameFiles::new(),
        path,
    )?)
}
