//! `gfx::palette_to_dwords` (0x405DDE): turns a palette of 256 entries of three bytes into
//! 256 dwords.
//!
//! In:  esi = the three-byte palette, edi = the dwords
//! Out: every register preserved
//! A dword is the entry's first byte shifted up 16, its second shifted up 8, and its third;
//! the top byte is 0. Entry 255 is done first. Four callers, the picture readers.
//! Refuses null pointers. The name is a reading.

use crate::errors::BadInput;
use crate::pointers::GamePtr;

pub const ENTRIES: usize = 256;

// ---- Rust version ----

pub fn palette_to_dwords(from: GamePtr, to: GamePtr) -> Result<(), BadInput> {
    if from.is_null() || to.is_null() {
        return Err(BadInput("no palette"));
    }
    // From the last entry down, as the original goes: it matters if the two overlap. The
    // bytes are read first (an entry's three bytes lie below its dword, so the order of the
    // writes cannot change a read that comes after it).
    let palette = from.read_bytes(ENTRIES * 3, "a palette")?;
    let mut dwords = Vec::with_capacity(ENTRIES * 4);
    for entry in 0..ENTRIES {
        // Converted: `rc2k_core::gfx::pixels::palette_entry`.
        let bytes = [
            palette[entry * 3],
            palette[entry * 3 + 1],
            palette[entry * 3 + 2],
        ];
        dwords.extend_from_slice(&rc2k_core::gfx::pixels::palette_entry(bytes).to_le_bytes());
    }
    to.write_bytes(&dwords, "a palette's dwords")
}
