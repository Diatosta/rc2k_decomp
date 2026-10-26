//! `files::copy_file` (0x402C28): a file copied, its time with it: the source opened to
//! read (`files::open_by_mode` 0), the destination created (mode 1), 64 KB at a time
//! through `g_pngInfo` (the PNG reader's buffer) until a read gives 0 bytes, then the
//! source's last write time (`sub_402F1C`: `GetFileTime`, `FileTimeToDosDateTime`) put on
//! the destination as its last access and last write time (`sub_402F4D`:
//! `DosDateTimeToFileTime`, `SetFileTime`; so to two seconds). A failed read, a short or
//! failed write or a failed time ends it; both files are closed (the destination first).
//! `sub_402F1C` and `sub_402F4D` are not hooked (each a Win32 call or two, as
//! `files/handle.rs`'s): `files::handle::{file_time, set_file_time}`, through std and
//! `rc2k_core::files::dos_time`.
//!
//! In:  eax = the source's path, ebx = the destination's (both through `resolve_path`)
//! Out: every register kept (`pusha` .. `popa`); carry clear if it was all done, set if
//!      not (zero set if done)
//!
//! Five callers: `install::run_after_exit`, `sub_4139E5`, `sub_41EA1F`, `sub_421B42`,
//! `sub_4599AD`. Refuses (before anything) what `open_by_mode` refuses (a resolved path
//! past its buffer).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::files::scratch::SCRATCH;
use crate::pointers::GamePtr;

const CHUNK: u32 = 0x1_0000;

/// The Rust version: whether it was all done.
pub fn copy_file(from: *const u8, to: *const u8) -> Result<bool, BadInput> {
    let scratch = GamePtr::new(SCRATCH.buffer.addr() as u32);
    let mut buffer = scratch.read_bytes(CHUNK as usize, "the scratch buffer")?;
    let done = rc2k_core::files::io::copy_file(
        &mut crate::files::handle::GameFiles::new(),
        GamePtr::new(from as u32),
        GamePtr::new(to as u32),
        &mut buffer,
    )?;
    scratch.write_bytes(&buffer, "the scratch buffer")?;
    Ok(done)
}
