//! `files::read_file` (0x402BB6): opens a file for reading, seeks if the offset is not 0,
//! reads, closes.
//!
//! Takes path, buffer, bytes to read, offset. Returns bytes read if the read was reached, else
//! unchanged. It fails if the open, the seek, the read or the close fails. A short read is not a
//! failure: the caller has to look at eax. The file is opened as the original's mode 0 says (table
//! at 0x4E06EA). Like the original's `open_by_mode` and `close`, which are inlined here, it sets
//! `g_lastFilePath` to the resolved path while the file is open and to 0 after closing; after a
//! failed open the path stays.

use crate::errors::BadInput;
// ---- Rust version ----

use crate::files::handle::GameFiles;
use crate::pointers::GamePtr;
pub use rc2k_core::files::io::ReadResult;

/// Refuses what `resolve_path` refuses.
#[track_caller]
pub fn read_file(
    path: &[u8],
    to: GamePtr,
    count: u32,
    offset: u32,
) -> Result<ReadResult, BadInput> {
    // Converted: `rc2k_core::files::io::read_file`.
    let mut into = to.read_bytes(count as usize, "a file's buffer")?;
    let result = rc2k_core::files::io::read_file(&mut GameFiles::new(), path, &mut into, offset)?;
    to.write_bytes(&into, "a file's buffer")?;
    Ok(result)
}

/// `read_file` into Rust's memory: `into` holds what the file's bytes do not reach over,
/// as the game's buffer did. Refuses what `resolve_path` refuses.
pub fn read_file_into(path: &[u8], into: &mut [u8], offset: u32) -> Result<ReadResult, BadInput> {
    Ok(rc2k_core::files::io::read_file(
        &mut GameFiles::new(),
        path,
        into,
        offset,
    )?)
}

/// `read_file` into a store's field from its first byte.
#[track_caller]
pub fn read_file_to_store(
    path: &[u8],
    to: &dyn crate::pointers::Store,
    count: u32,
    offset: u32,
) -> Result<ReadResult, BadInput> {
    let mut into = vec![0u8; count as usize];
    to.load(0, &mut into)?;
    let result = rc2k_core::files::io::read_file(&mut GameFiles::new(), path, &mut into, offset)?;
    to.store(0, &into)?;
    Ok(result)
}
