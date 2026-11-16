//! The next byte of a buffered stream.
//!
//! The buffer's bytes are handed out one by one; when it is used up it is filled again
//! (`files::read`, up to 0x2000 bytes). Nothing read is the end of the file; after a failed read
//! the stream is left as it was. The name is a reading.

use crate::errors::BadInput;
use crate::files::stream::Stream;
pub use rc2k_core::files::io::ByteRead;
pub fn read_stream_byte(stream: &mut Stream) -> Result<ByteRead, BadInput> {
    Ok(rc2k_core::files::io::read_stream_byte(
        &mut crate::files::handle::GameFiles::new(),
        stream,
    )?)
}
