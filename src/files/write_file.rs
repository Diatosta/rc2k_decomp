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

use crate::files::handle::GameFiles;
use crate::pointers::GamePtr;
pub use rc2k_core::files::io::WriteResult;

/// Refuses what `resolve_path` refuses.
pub fn write_file(path: *const u8, buffer: *const u8, count: u32) -> Result<WriteResult, BadInput> {
    // Converted: `rc2k_core::files::io::write_file`.
    let data = GamePtr::new(crate::pointers::game_address(buffer))
        .read_bytes(count as usize, "a file's contents")?;
    Ok(rc2k_core::files::io::write_file(
        &mut GameFiles::new(),
        GamePtr::new(crate::pointers::game_address(path)),
        &data,
    )?)
}
