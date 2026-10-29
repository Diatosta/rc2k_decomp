//! `files::close_stream` (0x402CFA): closes a buffered stream.
//!
//! In:  esi = the stream
//! Out: carry = the handle could not be closed (zero flag = it could); every register
//!      preserved
//! A stream opened with mode 1 has what is in its buffer written out first (`sub_402D8E`:
//! `filled` bytes, nothing if that is 0; whether the write worked is not looked at). Then
//! the handle is closed and -1 stored in its place. Five callers.
//! The name is a reading.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::files::handle::NO_HANDLE;
use crate::files::stream::Stream;

/// The Rust version.
pub fn close_stream(stream: &mut Stream) -> Result<bool, BadInput> {
    Ok(rc2k_core::files::io::close_stream(
        &mut crate::files::handle::GameFiles::<&[u8]>::new(),
        stream,
        NO_HANDLE,
    )?)
}
