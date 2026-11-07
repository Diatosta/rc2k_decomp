//! `files::read_stream_block` (0x402DA3): reads a number of bytes from a buffered stream.
//!
//! In:  ebx = where they go, ecx = how many, esi = the stream
//! Out: eax = how many were read; carry = a read failed; everything else preserved
//! One `read_stream_byte` per byte. The end of the file stops it without the carry flag:
//! the caller has to look at eax. Three callers, the picture readers (0x80 bytes of header).
//! The name is a reading.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::files::stream::Stream;

/// The Rust version: `to.len()` bytes into `to`; the bytes read and the original's flag.
pub fn read_stream_block(stream: &mut Stream, to: &mut [u8]) -> Result<(u32, bool), BadInput> {
    Ok(rc2k_core::files::io::read_stream_block(
        &mut crate::files::handle::GameFiles::new(),
        stream,
        to,
    )?)
}
