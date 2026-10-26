//! `strings::copy_printable` (0x4017F2): copies a name, leaving out what cannot be shown.
//!
//! In:  eax = the text (0: none), ebx = where to write, ecx = the room, NUL included
//! Out: every register preserved
//! At most room - 1 characters are looked at, up to the text's end. One whose low seven
//! bits are under 0x20 is left out; the others are written, in lower case through
//! `g_lowerCase` unless `dword_4E0144` is 1. A NUL ends what was written. Nine callers.
//! A room of 0 lets the original look at 2^32 - 1 characters: refused.
//! Manual patch with a private copy (0x37 bytes): short jumps in the first bytes.

use crate::errors::BadInput;
use crate::strings::same_text::case_table;
use std::sync::atomic::{AtomicU32, Ordering};

/// 1 = the case is kept: out of the image since 2026-10-24 (phase 5; it was `dword_4E0144`,
/// 1 there). `input::joystick_found` sets it to 0 for a while.
pub static KEEP_CASE: AtomicU32 = AtomicU32::new(1);

// ---- Rust version ----

/// What is written, with its NUL (converted: `rc2k_core::strings::case::CaseTable::printable`).
pub unsafe fn printable(text: *const u8, room: u32) -> Result<Vec<u8>, BadInput> {
    let mut looked_at = Vec::new();
    if !text.is_null() {
        // As far as the original reads: room - 1 characters or the NUL.
        for n in 0..(room as usize).saturating_sub(1) {
            let c = unsafe { text.add(n).read() };
            if c == 0 {
                break;
            }
            looked_at.push(c);
        }
    }
    let keep_case = KEEP_CASE.load(Ordering::Relaxed) == 1;
    case_table()
        .printable(&looked_at, room as usize, keep_case)
        .ok_or(BadInput("no room for a name: the original runs on"))
}

pub unsafe fn copy_printable(text: *const u8, to: *mut u8, room: u32) -> Result<(), BadInput> {
    unsafe {
        let bytes = printable(text, room)?;
        crate::pointers::GamePtr::new(to as usize as u32)
            .write_bytes(&bytes, "a printable name")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn switch_starts_as_the_games() {
        let d = crate::moved::image_bytes(0x4E0144, 4);
        assert_eq!(
            u32::from_le_bytes(d.try_into().unwrap()),
            super::KEEP_CASE.load(std::sync::atomic::Ordering::Relaxed)
        );
    }
}
