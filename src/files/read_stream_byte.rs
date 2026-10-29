//! `files::read_stream_byte` (0x402D16): the next byte of a buffered stream.
//!
//! In:  esi = the stream; eax as it comes
//! Out: al = the byte, carry clear; or carry set with ah = 0 at the end of the file (eax =
//!      0) and ah = 1 after a read error; the zero flag is clear in every case; everything
//!      but eax preserved
//! While the buffer has bytes left it hands out the next one and only al changes. Otherwise
//! it reads up to 0x2000 bytes into the buffer (`files::read`), and eax becomes the number
//! read: with at least one, the first is handed out in al; with none, that is the end of the
//! file. After a failed read the stream is left as it was.
//! Seven call sites: `read_stream_block` and the picture readers, which call it per byte.
//! The name is a reading.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::files::stream::Stream;
pub use rc2k_core::files::io::ByteRead;

/// The Rust version.
pub fn read_stream_byte(stream: &mut Stream, eax: u32) -> Result<ByteRead, BadInput> {
    Ok(rc2k_core::files::io::read_stream_byte(
        &mut crate::files::handle::GameFiles::<&[u8]>::new(),
        stream,
        eax,
    )?)
}
