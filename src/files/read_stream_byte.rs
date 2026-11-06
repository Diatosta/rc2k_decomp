//! `files::read_stream_byte` (0x402D16): the next byte of a buffered stream.
//!
//! The buffer's bytes are handed out one by one; when it is used up it is filled again
//! (`files::read`, up to 0x2000 bytes). Nothing read is the end of the file; after a failed
//! read the stream is left as it was. Used by `read_stream_block` and the picture readers.
//! The name is a reading.

use crate::errors::BadInput;
use crate::files::stream::Stream;
pub use rc2k_core::files::io::ByteRead;

/// The Rust version.
pub fn read_stream_byte(stream: &mut Stream) -> Result<ByteRead, BadInput> {
    Ok(rc2k_core::files::io::read_stream_byte(
        &mut crate::files::handle::GameFiles::<&[u8]>::new(),
        stream,
    )?)
}
