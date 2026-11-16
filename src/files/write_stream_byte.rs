//! A byte into a buffered stream opened to write
//! (the block `files/stream.rs` describes): a full buffer (position 0x2000 or more) first
//! written out (`files::write`, the bytes in it) and emptied; the byte at the position,
//! position and count one more.

use crate::errors::BadInput;
use crate::files::stream::Stream;
pub fn write_stream_byte(stream: &mut Stream, eax: u32) -> Result<bool, BadInput> {
    Ok(rc2k_core::files::io::write_stream_byte(
        &mut crate::files::handle::GameFiles::new(),
        stream,
        eax as u8,
    )?)
}
