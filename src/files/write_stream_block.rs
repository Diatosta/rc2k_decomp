//! The given bytes into a buffered stream, one by one (`files::write_stream_byte`), until one
//! fails.

use crate::errors::BadInput;
use crate::files::stream::Stream;

/// All of `bytes` written: whether it worked.
pub fn write_stream_all(stream: &mut Stream, bytes: &[u8]) -> Result<bool, BadInput> {
    Ok(rc2k_core::files::io::write_stream_all(
        &mut crate::files::handle::GameFiles::new(),
        stream,
        bytes,
    )?)
}
