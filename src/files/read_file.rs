//! `files::read_file` (0x402BB6): opens a file for reading, seeks if the offset is not 0,
//! reads, closes.
//!
//! In:  eax = path, ebx = buffer, ecx = bytes to read, edx = offset
//! Out: carry = failed (zero flag = succeeded); eax = bytes read if the read was reached,
//!      else unchanged; everything else preserved
//! It fails if the open, the seek, the read or the close fails. A short read is not a failure:
//! the caller has to look at eax. The file is opened as the original's mode 0 says (table at
//! 0x4E06EA). Like the original's `open_by_mode` and `close`, which are inlined here, it sets
//! `g_lastFilePath` to the resolved path while the file is open and to 0 after closing; after
//! a failed open the path stays.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::rva;

// ---- game data ----

pub const LAST_FILE_PATH_RVA: usize = rva(0x4F52B0); // g_lastFilePath

// ---- Rust version ----

use crate::files::handle::GameFiles;
pub use rc2k_core::files::io::ReadResult;

/// Refuses what `resolve_path` refuses.
pub unsafe fn read_file(
    path: *const u8,
    buffer: *mut u8,
    count: u32,
    offset: u32,
) -> Result<ReadResult, BadInput> {
    // Converted: `rc2k_core::files::io::read_file`.
    unsafe {
        let into = std::slice::from_raw_parts_mut(buffer, count as usize);
        Ok(rc2k_core::files::io::read_file(
            &mut GameFiles::new(),
            path,
            into,
            offset,
        )?)
    }
}
