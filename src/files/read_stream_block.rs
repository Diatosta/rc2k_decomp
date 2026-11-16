//! Reads a number of bytes from a buffered stream.
//!
//! Gives how many were read. One `read_stream_byte` per byte. The end of the file stops it without
//! the failure flag: the caller has to look at the count.

use crate::errors::BadInput;
use crate::files::stream::Stream;

/// Reads `to.len()` bytes into `to`; gives the bytes read and the flag.
pub fn read_stream_block(stream: &mut Stream, to: &mut [u8]) -> Result<(u32, bool), BadInput> {
    Ok(rc2k_core::files::io::read_stream_block(
        &mut crate::files::handle::GameFiles::new(),
        stream,
        to,
    )?)
}
