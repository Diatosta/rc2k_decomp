//! `files::open_stream` (0x402CDD): opens a file as a buffered stream.
//!
//! In:  eax = path, ebx = file mode, esi = the stream
//! Out: carry = the file could not be opened; every register preserved
//! Stores the mode in the stream, opens the file with `open_by_mode` and, if that worked,
//! stores the handle and an empty buffer (position 0, 0 bytes). After a failure only the
//! mode is stored. Five callers, the picture readers.
//! Refuses what `open_by_mode` refuses. The name is a reading.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::files::stream::Stream;

// ---- Rust version ----

/// Returns whether the file was opened.
/// The Rust version: whether the file `path` opened.
pub fn open_stream(stream: &mut Stream, path: &[u8], mode: u32) -> Result<bool, BadInput> {
    // Converted: `rc2k_core::files::io::open_stream`.
    Ok(rc2k_core::files::io::open_stream(
        &mut crate::files::handle::GameFiles::<&[u8]>::new(),
        stream,
        path,
        mode,
    )?)
}
