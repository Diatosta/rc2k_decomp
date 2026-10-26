//! `files::file_exists` (0x402EBE): opens a file for reading and closes it again.
//!
//! In:  eax = path
//! Out: carry = it could not be opened (or not closed); zero flag = it could; every register
//!      preserved
//! `open_by_mode` with mode 0, then `close`. `g_lastFilePath` is 0 after a file that
//! exists and the resolved path after one that does not. 20 callers.
//! Refuses what `resolve_path` refuses. The name is a reading.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;

// ---- Rust version ----

pub fn file_exists(path: *const u8) -> Result<bool, BadInput> {
    Ok(rc2k_core::files::io::file_exists(
        &mut crate::files::handle::GameFiles::new(),
        crate::pointers::GamePtr::new(crate::pointers::game_address(path)),
    )?)
}
