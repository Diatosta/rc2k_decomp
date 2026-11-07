//! `files::write_file` (0x402BFB): creates a file (replacing one that exists), writes a
//! buffer to it, closes it.
//!
//! In:  eax = path, ebx = buffer, ecx = bytes to write
//! Out: carry = failed (zero flag = succeeded); eax = bytes written if the write was reached,
//!      else unchanged; everything else preserved
//! It fails if the open fails, if the write fails or writes less than asked, or if the close
//! fails. It does not create missing folders. The file is opened as the original's mode 1
//! says (table at 0x4E06EA). `open_by_mode`, `write` and `close` are inlined, with their
//! handling of `g_lastFilePath`.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;

// ---- game data ----

// ---- Rust version ----

use crate::files::handle::{GameFiles, GamePath};
pub use rc2k_core::files::io::WriteResult;

/// The file written from `data`. Refuses what `resolve_path` refuses.
pub fn write_file_from(path: impl GamePath, data: &[u8]) -> Result<WriteResult, BadInput> {
    // Converted: `rc2k_core::files::io::write_file`.
    Ok(rc2k_core::files::io::write_file(
        &mut GameFiles::new(),
        path,
        data,
    )?)
}

/// `write_file` from the first `count` bytes of a store's field.
pub fn write_file_from_store(
    path: impl GamePath,
    from: &dyn crate::pointers::Store,
    count: u32,
) -> Result<WriteResult, BadInput> {
    let mut data = vec![0u8; count as usize];
    from.load(0, &mut data)?;
    write_file_from(path, &data)
}
