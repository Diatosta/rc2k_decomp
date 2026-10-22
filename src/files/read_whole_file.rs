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
use crate::files::handle::NO_HANDLE;
use crate::framework::{EAX, EBX, game_write, rva};
use crate::screens::harness::Left;

// ---- game data ----

pub const BYTES_READ_RVA: usize = rva(0x60B2CC); // dword_60B2CC

// ---- Rust version ----

/// The bytes read (0 after any failure but a failed read's own count).
pub unsafe fn read_whole_file(
    path: *const u8,
    buffer: *mut u8,
    count: u32,
    ebx: u32,
) -> Result<Left, BadInput> {
    // Converted: `rc2k_core::files::io::read_whole_file`.
    unsafe {
        let into = std::slice::from_raw_parts_mut(buffer, count as usize);
        let w = rc2k_core::files::io::read_whole_file(
            &mut crate::files::handle::GameFiles::new(),
            path,
            into,
        )?;
        if !w.opened {
            game_write(BYTES_READ_RVA, 0u32);
            return Ok(Left::carry(true).with(EAX, NO_HANDLE));
        }
        game_write(BYTES_READ_RVA, if w.closed { w.bytes } else { 0 });
        Ok(Left::carry(!w.closed || !w.read_ok)
            .with(EAX, w.closed as u32)
            .with(EBX, (ebx & !0xFF) | w.read_ok as u32))
    }
}
