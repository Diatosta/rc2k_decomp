//! `gfx::palette_to_dwords` (0x405DDE): turns a palette of 256 entries of three bytes into
//! 256 dwords.
//!
//! In:  esi = the three-byte palette, edi = the dwords
//! Out: every register preserved
//! A dword is the entry's first byte shifted up 16, its second shifted up 8, and its third;
//! the top byte is 0. Entry 255 is done first. Four callers, the picture readers.
//! Refuses null pointers. The name is a reading.

use crate::errors::BadInput;

pub const ENTRIES: usize = 256;

// ---- Rust version ----

pub unsafe fn palette_to_dwords(from: *const u8, to: *mut u32) -> Result<(), BadInput> {
    if from.is_null() || to.is_null() {
        return Err(BadInput("no palette"));
    }
    // From the last entry down, as the original goes: it matters if the two overlap.
    for entry in (0..ENTRIES).rev() {
        unsafe {
            // Converted: `rc2k_core::gfx::pixels::palette_entry`.
            let bytes = (from.add(entry * 3) as *const [u8; 3]).read();
            to.add(entry)
                .write_unaligned(rc2k_core::gfx::pixels::palette_entry(bytes));
        }
    }
    Ok(())
}
