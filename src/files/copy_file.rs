//! A file copied, its time with it: the source opened to read, the destination created, 64 KB at
//! a time until a read gives 0 bytes, then the source's last write time put on the destination
//! (to two seconds, as DOS time). A failed read, a short or failed write or a failed time ends
//! it; both files are closed (the destination first). Refuses what `open_by_mode` refuses.

use crate::errors::BadInput;
use crate::system::owner::System;

const CHUNK: u32 = 0x1_0000;

/// Whether it was all done.
/// The paths without their NULs.
pub fn copy_file(system: &System, from: &[u8], to: &[u8]) -> Result<bool, BadInput> {
    let mut buffer = system
        .work()
        .files
        .buffer
        .read_at(0, CHUNK as usize, "the scratch buffer")?;
    let done = rc2k_core::files::io::copy_file(
        &mut crate::files::handle::GameFiles::new(),
        from,
        to,
        &mut buffer,
    )?;
    system
        .work()
        .files
        .buffer
        .write_at(0, &buffer, "the scratch buffer")?;
    Ok(done)
}
