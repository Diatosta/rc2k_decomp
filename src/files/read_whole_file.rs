//! `files::read_whole_file` (0x421B54): reads a file's start into a buffer and says how
//! much it got.
//!
//! In:  eax = the path, ebx = the buffer, ecx = how many bytes at most
//! Out: carry set = the file could not be opened, read or closed (zero clear after a
//!      failed read); `dword_60B2CC` = the bytes read, 0 after a failed open or close;
//!      eax = what `CloseHandle` gave, or -1 if the file did not open; bl = 1 after a
//!      good read, 0 after a failed one; everything else preserved
//!
//! `files::open_by_mode` with mode 0, `files::read`, `files::close`.
//! 29 callers: the models, the stage's files.
//! Refuses what `files::resolve_path` refuses.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::files::state::BYTES_READ;

// ---- Rust version ----

/// Reads the whole file at `path` into `buffer` (`count` bytes at most). The bytes read
/// go to `BYTES_READ` (0 after any failure but a failed read's own count).
pub unsafe fn read_whole_file(
    path: *const u8,
    buffer: *mut u8,
    count: u32,
) -> Result<(), BadInput> {
    // Converted: `rc2k_core::files::io::read_whole_file`.
    unsafe {
        let into = std::slice::from_raw_parts_mut(buffer, count as usize);
        let w = rc2k_core::files::io::read_whole_file(
            &mut crate::files::handle::GameFiles::new(),
            path,
            into,
        )?;
        if !w.opened {
            BYTES_READ.store(0, std::sync::atomic::Ordering::Relaxed);
            return Ok(());
        }
        BYTES_READ.store(
            if w.closed { w.bytes } else { 0 },
            std::sync::atomic::Ordering::Relaxed,
        );
        Ok(())
    }
}
