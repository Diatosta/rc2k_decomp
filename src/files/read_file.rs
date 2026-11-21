//! Opens a file for reading, seeks if the offset is not 0, reads, closes. It fails if the open,
//! the seek, the read or the close fails; a short read is not a failure (the caller looks at the
//! count). The last file path is the resolved path while the file is open and none after the
//! close; after a failed open it stays.

use crate::errors::BadInput;
use crate::files::handle::GameFiles;
pub use rc2k_core::files::io::ReadResult;

/// `read_file` into Rust's memory: `into` holds what the file's bytes do not reach over,
/// as the game's buffer did. Refuses what `resolve_path` refuses.
pub fn read_file_into(path: &[u8], into: &mut [u8], offset: u32) -> Result<ReadResult, BadInput> {
    Ok(rc2k_core::files::io::read_file(
        &mut GameFiles::new(),
        path,
        into,
        offset,
    )?)
}

/// `read_file` into a buffer's first `count` bytes (more than it holds is refused).
#[track_caller]
pub fn read_file_to_buffer<const N: usize>(
    path: &[u8],
    to: &crate::memory::plain_buffer::PlainBuffer<N>,
    count: u32,
    offset: u32,
) -> Result<ReadResult, BadInput> {
    let mut into = to.read_at(0, count as usize, "a file's buffer")?;
    let result = rc2k_core::files::io::read_file(&mut GameFiles::new(), path, &mut into, offset)?;
    to.write_at(0, &into, "a file's buffer")?;
    Ok(result)
}
