//! Opens a file as a buffered stream.
//!
//! Stores the mode in the stream, opens the file with `open_by_mode` and, if that worked, stores
//! the handle and an empty buffer (position 0, 0 bytes). After a failure only the mode is stored.
//! Used by the picture readers. Refuses what `open_by_mode` refuses.

use crate::errors::BadInput;
use crate::files::stream::Stream;

/// Returns whether the file was opened.
/// Whether the file `path` opened.
pub fn open_stream(stream: &mut Stream, path: &[u8], mode: u32) -> Result<bool, BadInput> {
    Ok(rc2k_core::files::io::open_stream(
        &mut crate::files::handle::GameFiles::new(),
        stream,
        path,
        mode,
    )?)
}
