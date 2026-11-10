//! `files::copy_file` (0x402C28): a file copied, its time with it: the source opened to
//! read (`files::open_by_mode` 0), the destination created (mode 1), 64 KB at a time
//! through `g_pngInfo` (the PNG reader's buffer) until a read gives 0 bytes, then the
//! source's last write time (`files::file_time`: `GetFileTime`, `FileTimeToDosDateTime`) put on
//! the destination as its last access and last write time (`files::set_file_time`:
//! `DosDateTimeToFileTime`, `SetFileTime`; so to two seconds). A failed read, a short or
//! failed write or a failed time ends it; both files are closed (the destination first).
//! `files::file_time` and `files::set_file_time` are not separate functions (each a Win32 call or
//! two, as
//! `files/handle.rs`'s): `files::handle::{file_time, set_file_time}`, through std and
//! `rc2k_core::files::dos_time`.
//!
//! Five callers: `install::run_after_exit`, 0x4139E5, 0x41EA1F, 0x421B42, 0x4599AD.
//! Refuses (before anything) what `open_by_mode` refuses (a resolved path past its buffer).

use crate::errors::BadInput;
use crate::system::owner::system;

const CHUNK: u32 = 0x1_0000;

/// Whether it was all done.
/// The paths without their NULs.
pub fn copy_file(from: &[u8], to: &[u8]) -> Result<bool, BadInput> {
    let system = system();
    let mut buffer = system
        .work
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
        .work
        .files
        .buffer
        .write_at(0, &buffer, "the scratch buffer")?;
    Ok(done)
}
