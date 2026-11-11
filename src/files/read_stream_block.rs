//! `files::read_stream_block` (0x402DA3): reads a number of bytes from a buffered stream.
//!
//! Returns how many were read. One `read_stream_byte` per byte. The end of the file stops it
//! without the carry flag: the caller has to look at the count. Three callers, the picture readers (0x80
//! bytes of header). The name is a reading.

use crate::errors::BadInput;
use crate::files::stream::Stream;

/// `to.len()` bytes into `to`; the bytes read and the original's flag.
pub fn read_stream_block(stream: &mut Stream, to: &mut [u8]) -> Result<(u32, bool), BadInput> {
    Ok(rc2k_core::files::io::read_stream_block(
        &mut crate::files::handle::GameFiles::new(),
        stream,
        to,
    )?)
}
