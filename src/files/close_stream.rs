//! Closes a buffered stream: a stream opened with mode 1 has its buffer written out first (the
//! write's result is not looked at), then the handle is closed and -1 stored in its place.

use crate::errors::BadInput;
use crate::files::handle::NO_HANDLE;
use crate::files::stream::Stream;
pub fn close_stream(stream: &mut Stream) -> Result<bool, BadInput> {
    Ok(rc2k_core::files::io::close_stream(
        &mut crate::files::handle::GameFiles::new(),
        stream,
        NO_HANDLE,
    )?)
}
